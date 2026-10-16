import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, readFile, readdir, rename, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { stableJson } from "../../shared/hash";
import {
  busJob,
  channelJob,
  jobArgs,
  loudnessMeasureJob,
  masterJob,
  measureArgs,
  parseLoudnorm,
  type Job,
} from "../../shared/mix/graph";
import { channelKey, hasSound, type MixPlan } from "../../shared/mix/plan";
import type { Loudness, MixResult, StemLevels } from "../../shared/mix/result";
import type { FfmpegTools } from "../media/ffmpeg";
import { frameLevels, readWav } from "./wav";

export const MIX_VERSION = "demoflow-mix/2";
const KEEP_PLANS = 12;

export type MixDeps = {
  projectDir: string;
  resolve: (ref: string) => string;
  ffmpeg: FfmpegTools;
  signal?: AbortSignal;
  onProgress?: (progress: number, message: string) => void;
};

export const mixDir = (projectDir: string) => path.join(projectDir, ".demoflow", "cache", "mix");

const sha = (value: unknown) =>
  createHash("sha256")
    .update(MIX_VERSION)
    .update("\0")
    .update(stableJson(value))
    .digest("hex")
    .slice(0, 24);

async function identity(file: string): Promise<string> {
  try {
    const s = await stat(file);
    return `${s.size}:${Math.round(s.mtimeMs)}`;
  } catch {
    return "missing";
  }
}

type Stem = { hash: string; file: string; levels: StemLevels; reduction: number[] | null };

async function readJson<T>(file: string): Promise<T | null> {
  try {
    return JSON.parse(await readFile(file, "utf8")) as T;
  } catch {
    return null;
  }
}

async function runStem(
  deps: MixDeps,
  plan: MixPlan,
  hash: string,
  job: Job,
  codec: "pcm_f32le" | "pcm_s16le",
  makeup = 0,
): Promise<Stem> {
  const dir = mixDir(deps.projectDir);
  const file = path.join(dir, `${hash}.wav`);
  const meta = path.join(dir, `${hash}.json`);
  const cached = existsSync(file) ? await readJson<Omit<Stem, "file" | "hash">>(meta) : null;
  if (cached) return { hash, file, ...cached };
  await mkdir(dir, { recursive: true });
  const tmp = `${file}.${process.pid}.partial.wav`;
  const taps = job.taps
    ? { pre: `${file}.${process.pid}.pre.wav`, post: `${file}.${process.pid}.post.wav` }
    : undefined;
  try {
    await deps.ffmpeg.run(jobArgs(job, tmp, codec, taps, plan.durationInFrames / plan.fps + 1), {
      signal: deps.signal,
    });
    const levels = frameLevels(await readWav(tmp), plan.fps, plan.durationInFrames);
    let reduction: number[] | null = null;
    if (taps) {
      const pre = frameLevels(await readWav(taps.pre), plan.fps, plan.durationInFrames);
      const post = frameLevels(await readWav(taps.post), plan.fps, plan.durationInFrames);
      reduction = pre.rms.map((v, i) =>
        v <= -100 ? 0 : Math.max(0, Math.round((v - (post.rms[i] - makeup)) * 10) / 10),
      );
    }
    await rename(tmp, file);
    await writeFile(meta, JSON.stringify({ levels, reduction }));
    return { hash, file, levels, reduction };
  } finally {
    await rm(tmp, { force: true });
    if (taps) {
      await rm(taps.pre, { force: true });
      await rm(taps.post, { force: true });
    }
  }
}

async function loudnessOf(deps: MixDeps, plan: MixPlan, master: Stem): Promise<Loudness> {
  const file = path.join(mixDir(deps.projectDir), `${master.hash}.loudness.json`);
  const cached = await readJson<Loudness>(file);
  if (cached) return cached;
  const lines: string[] = [];
  await deps.ffmpeg.run(
    [
      "-loglevel",
      "verbose",
      "-i",
      master.file,
      "-af",
      "ebur128=peak=true:framelog=verbose",
      "-f",
      "null",
      "-",
    ],
    { signal: deps.signal, onLog: (line) => lines.push(line) },
  );
  const loudness = parseEbur128(lines, plan.fps, plan.durationInFrames);
  await writeFile(file, JSON.stringify(loudness));
  return loudness;
}

const FRAME_LINE =
  /t:\s*([\d.]+)\s+TARGET:\S+\s+LUFS\s+M:\s*(-?[\d.]+|-?inf)\s+S:\s*(-?[\d.]+|-?inf)\s+I:\s*(-?[\d.]+|-?inf)\s+LUFS\s+LRA:\s*([\d.]+)/;

export function parseEbur128(lines: string[], fps: number, frames: number): Loudness {
  const samples: Array<{ t: number; m: number; s: number }> = [];
  let integrated: number | null = null;
  let range: number | null = null;
  let truePeak: number | null = null;
  let section = "";
  const num = (v: string) => {
    const n = Number(v);
    return Number.isFinite(n) ? n : -120;
  };
  for (const line of lines) {
    const m = FRAME_LINE.exec(line);
    if (m) {
      samples.push({ t: Number(m[1]), m: num(m[2]), s: num(m[3]) });
      continue;
    }
    const text = line.replace(/^\[[^\]]*\]\s*/, "").trim();
    if (text.startsWith("Integrated loudness")) section = "i";
    else if (text.startsWith("Loudness range")) section = "lra";
    else if (text.startsWith("True peak")) section = "tp";
    const value = /^(I|LRA|Peak):\s*(-?[\d.]+|-?inf)/.exec(text);
    if (!value) continue;
    if (value[1] === "I" && section === "i") integrated = num(value[2]);
    else if (value[1] === "LRA" && section === "lra") range = num(value[2]);
    else if (value[1] === "Peak" && section === "tp") truePeak = num(value[2]);
  }
  const momentary = Array.from({ length: frames }, () => -120);
  const shortTerm = Array.from({ length: frames }, () => -120);
  let k = -1;
  for (let f = 0; f < frames; f++) {
    const at = (f + 1) / fps;
    while (k + 1 < samples.length && samples[k + 1].t <= at + 1e-6) k++;
    if (k < 0) continue;
    momentary[f] = Math.round(samples[k].m * 10) / 10;
    shortTerm[f] = Math.round(samples[k].s * 10) / 10;
  }
  const round = (v: number | null) => (v === null || v <= -70 ? null : Math.round(v * 10) / 10);
  return {
    fps,
    momentary,
    shortTerm,
    integrated: round(integrated),
    range: range === null ? null : Math.round(range * 10) / 10,
    truePeak: truePeak === null || truePeak <= -120 ? null : Math.round(truePeak * 10) / 10,
  };
}

export function friendlyMixError(error: unknown): Error {
  const text = error instanceof Error ? error.message : String(error);
  if (/cancel/i.test(text)) return new Error("Cancelled");
  const missing =
    /Error opening input file .*?([^/\\]+?)\.?$/m.exec(text) ??
    /([^/\\\s]+\.\w+): No such file or directory/.exec(text);
  if (missing) return new Error(`The sound file “${missing[1]}” is missing from the project.`);
  return new Error("The sound couldn't be mixed. Check that every sound file still opens.");
}

export async function renderMix(plan: MixPlan, deps: MixDeps): Promise<MixResult> {
  try {
    return await renderMixStems(plan, deps);
  } catch (error) {
    throw friendlyMixError(error);
  }
}

async function renderMixStems(plan: MixPlan, deps: MixDeps): Promise<MixResult> {
  const empty: MixResult = {
    key: plan.key,
    master: null,
    levels: { channels: {}, buses: {}, master: null },
    reduction: { channels: {}, master: null },
    loudness: null,
  };
  if (!hasSound(plan)) return empty;
  const progress = deps.onProgress ?? (() => undefined);
  const steps = plan.channels.length + plan.buses.length + 2;
  let done = 0;
  const tick = (message: string) => progress(done++ / steps, message);

  const channels = new Map<string, Stem>();
  for (const channel of plan.channels) {
    tick("Mixing sound");
    const ids = await Promise.all(channel.sounds.map((s) => identity(deps.resolve(s.src))));
    const hash = `ch-${sha({ key: channelKey(plan, channel), ids })}`;
    channels.set(
      channel.id,
      await runStem(
        deps,
        plan,
        hash,
        channelJob(plan, channel, deps.resolve),
        "pcm_f32le",
        channel.strip.comp.on ? channel.strip.comp.makeup : 0,
      ),
    );
  }
  const stemOf = (id: string) => channels.get(id)!.file;
  const buses = new Map<string, Stem>();
  for (const bus of plan.buses) {
    tick("Mixing sound");
    const members = bus.channels.filter((c) => channels.has(c));
    const keys = bus.duck?.keys.filter((c) => channels.has(c)) ?? [];
    const busPlan = {
      ...bus,
      channels: members,
      duck: bus.duck && keys.length ? { ...bus.duck, keys } : null,
    };
    const hash = `bus-${sha({
      fps: plan.fps,
      frames: plan.durationInFrames,
      bus: { ...busPlan, name: undefined },
      stems: members.map((c) => channels.get(c)!.hash),
      keys: keys.map((c) => channels.get(c)!.hash),
    })}`;
    buses.set(bus.id, await runStem(deps, plan, hash, busJob(plan, busPlan, stemOf), "pcm_f32le"));
  }
  const busFiles = plan.buses.map((b) => buses.get(b.id)!.file);
  const busHashes = plan.buses.map((b) => buses.get(b.id)!.hash);
  tick("Balancing loudness");
  const masterHash = `master-${sha({ fps: plan.fps, frames: plan.durationInFrames, master: plan.master, buses: busHashes })}`;
  let measured = null;
  const masterFile = path.join(mixDir(deps.projectDir), `${masterHash}.wav`);
  if (!existsSync(masterFile)) {
    const measure = loudnessMeasureJob(plan, busFiles);
    if (measure) {
      const lines: string[] = [];
      await deps.ffmpeg.run(measureArgs(measure), {
        signal: deps.signal,
        onLog: (line) => lines.push(line),
      });
      measured = parseLoudnorm(lines.join("\n"));
    }
  }
  const master = await runStem(
    deps,
    plan,
    masterHash,
    masterJob(plan, busFiles, measured),
    "pcm_s16le",
    plan.master.comp.on ? plan.master.comp.makeup : 0,
  );
  tick("Measuring loudness");
  const loudness = await loudnessOf(deps, plan, master);
  await remember(deps.projectDir, [
    ...[...channels.values()].map((s) => s.hash),
    ...busHashes,
    master.hash,
  ]);
  progress(1, "Sound mixed");
  return {
    key: plan.key,
    master: {
      src: `project:${path.relative(deps.projectDir, master.file).split(path.sep).join("/")}`,
      hash: master.hash,
    },
    levels: {
      channels: Object.fromEntries([...channels].map(([id, s]) => [id, s.levels])),
      buses: Object.fromEntries([...buses].map(([id, s]) => [id, s.levels])),
      master: master.levels,
    },
    reduction: {
      channels: Object.fromEntries(
        [...channels].filter(([, s]) => s.reduction).map(([id, s]) => [id, s.reduction!]),
      ),
      master: master.reduction,
    },
    loudness,
  };
}

async function remember(projectDir: string, hashes: string[]): Promise<void> {
  const dir = mixDir(projectDir);
  const indexFile = path.join(dir, "recent.json");
  const recent = (await readJson<string[][]>(indexFile)) ?? [];
  const next = [hashes, ...recent.filter((h) => h.join() !== hashes.join())].slice(0, KEEP_PLANS);
  await writeFile(indexFile, JSON.stringify(next));
  const keep = new Set(next.flat());
  const cutoff = Date.now() - 10 * 60_000;
  for (const file of await readdir(dir).catch(() => [] as string[])) {
    if (file === "recent.json") continue;
    const hash = file.split(".")[0];
    if (keep.has(hash)) continue;
    const full = path.join(dir, file);
    const info = await stat(full).catch(() => null);
    if (info && info.mtimeMs < cutoff) await rm(full, { force: true });
  }
}
