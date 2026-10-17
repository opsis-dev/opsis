import { existsSync } from "node:fs";
import { mkdir, readdir, rename, rm, stat } from "node:fs/promises";
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
import { ToolError, type FfmpegTools } from "../media/ffmpeg";
import { frameLevels, readWav, SILENCE_DB } from "./wav";
import { errorMessage } from "../../shared/errors";
import { round1 } from "../../shared/math";
import { sha256Hex } from "../../engine/hash";
import { atomicWrite, readJson } from "../../engine/json";
import { projectPaths, toProjectRef } from "../../engine/project-paths";

export const MIX_VERSION = "demoflow-mix/1";
const KEEP_PLANS = 12;
const UNUSED_STEM_MS = 10 * 60_000;
const LOUDNESS_FLOOR_DB = -70;

export type MixDeps = {
  projectDir: string;
  resolve: (ref: string) => string;
  ffmpeg: FfmpegTools;
  signal?: AbortSignal;
  onProgress?: (progress: number, message: string) => void;
};

const mixDir = (projectDir: string) => path.join(projectPaths(projectDir).cache, "mix");

const sha = (value: unknown) => sha256Hex(`${MIX_VERSION}\0${stableJson(value)}`, 24);

async function identity(file: string): Promise<string> {
  try {
    const s = await stat(file);
    return `${s.size}:${Math.round(s.mtimeMs)}`;
  } catch {
    return "missing";
  }
}

type Stem = { hash: string; file: string; levels: StemLevels; reduction: number[] | null };

async function runStem(
  deps: MixDeps,
  plan: MixPlan,
  hash: string,
  makeJob: () => Job | Promise<Job>,
  codec: "pcm_f32le" | "pcm_s16le",
  makeup = 0,
): Promise<Stem> {
  const dir = mixDir(deps.projectDir);
  const file = path.join(dir, `${hash}.wav`);
  const meta = path.join(dir, `${hash}.json`);
  const cached = existsSync(file) ? await readJson<Omit<Stem, "file" | "hash">>(meta) : null;
  if (cached) return { hash, file, ...cached };
  await mkdir(dir, { recursive: true });
  const job = await makeJob();
  const tmp = `${file}.${process.pid}.partial.wav`;
  const taps = job.taps
    ? { pre: `${file}.${process.pid}.pre.wav`, post: `${file}.${process.pid}.post.wav` }
    : undefined;
  const levelsOf = async (wav: string) =>
    frameLevels(await readWav(wav), plan.fps, plan.durationInFrames);
  try {
    await deps.ffmpeg.run(jobArgs(job, tmp, codec, taps, plan.durationInFrames / plan.fps + 1), {
      signal: deps.signal,
    });
    const levels = await levelsOf(tmp);
    let reduction: number[] | null = null;
    if (taps) {
      const pre = await levelsOf(taps.pre);
      const post = await levelsOf(taps.post);
      reduction = pre.rms.map((v, i) =>
        v <= -100 ? 0 : Math.max(0, round1(v - (post.rms[i] - makeup))),
      );
    }
    await rename(tmp, file);
    await atomicWrite(meta, JSON.stringify({ levels, reduction }));
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
  await atomicWrite(file, JSON.stringify(loudness));
  return loudness;
}

const FRAME_LINE =
  /t:\s*([\d.]+)\s+TARGET:\S+\s+LUFS\s+M:\s*(-?[\d.]+|-?inf)\s+S:\s*(-?[\d.]+|-?inf)\s+I:\s*(-?[\d.]+|-?inf)\s+LUFS\s+LRA:\s*([\d.]+)/;

const SUMMARY_SECTIONS = [
  { heading: "Integrated loudness", label: "I", key: "integrated" },
  { heading: "Loudness range", label: "LRA", key: "range" },
  { heading: "True peak", label: "Peak", key: "truePeak" },
] as const;

type Summary = Record<(typeof SUMMARY_SECTIONS)[number]["key"], number | null>;

const decibels = (v: string) => {
  const n = Number(v);
  return Number.isFinite(n) ? n : SILENCE_DB;
};

export function parseEbur128(lines: string[], fps: number, frames: number): Loudness {
  const samples: Array<{ t: number; m: number; s: number }> = [];
  const summary: Summary = { integrated: null, range: null, truePeak: null };
  let section: (typeof SUMMARY_SECTIONS)[number] | undefined;
  for (const line of lines) {
    const m = FRAME_LINE.exec(line);
    if (m) {
      samples.push({ t: Number(m[1]), m: decibels(m[2]), s: decibels(m[3]) });
      continue;
    }
    const text = line.replace(/^\[[^\]]*\]\s*/, "").trim();
    section = SUMMARY_SECTIONS.find((s) => text.startsWith(s.heading)) ?? section;
    const value = /^(I|LRA|Peak):\s*(-?[\d.]+|-?inf)/.exec(text);
    if (value && section && value[1] === section.label) summary[section.key] = decibels(value[2]);
  }
  const momentary = Array.from({ length: frames }, () => SILENCE_DB);
  const shortTerm = Array.from({ length: frames }, () => SILENCE_DB);
  let k = -1;
  for (let f = 0; f < frames; f++) {
    const at = (f + 1) / fps;
    while (k + 1 < samples.length && samples[k + 1].t <= at + 1e-6) k++;
    if (k < 0) continue;
    momentary[f] = round1(samples[k].m);
    shortTerm[f] = round1(samples[k].s);
  }
  const { integrated, range, truePeak } = summary;
  return {
    fps,
    momentary,
    shortTerm,
    integrated: integrated === null || integrated <= LOUDNESS_FLOOR_DB ? null : round1(integrated),
    range: range === null ? null : round1(range),
    truePeak: truePeak === null || truePeak <= SILENCE_DB ? null : round1(truePeak),
  };
}

export function friendlyMixError(error: unknown): Error {
  const text = error instanceof ToolError ? error.output : errorMessage(error);
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

type Tick = (message: string) => void;

async function channelStems(plan: MixPlan, deps: MixDeps, tick: Tick) {
  const channels = new Map<string, Stem>();
  for (const channel of plan.channels) {
    tick("Mixing sound");
    const ids = await Promise.all(channel.sounds.map((s) => identity(deps.resolve(s.src))));
    const hash = `ch-${sha({ key: channelKey(plan, channel), ids })}`;
    const makeup = channel.strip.comp.on ? channel.strip.comp.makeup : 0;
    const job = () => channelJob(plan, channel, deps.resolve);
    channels.set(channel.id, await runStem(deps, plan, hash, job, "pcm_f32le", makeup));
  }
  return channels;
}

async function busStems(plan: MixPlan, deps: MixDeps, channels: Map<string, Stem>, tick: Tick) {
  const stem = (id: string) => {
    const found = channels.get(id);
    if (!found) throw new Error(`The sound channel ${id} wasn't mixed.`);
    return found;
  };
  const buses: Array<[string, Stem]> = [];
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
      stems: members.map((c) => stem(c).hash),
      keys: keys.map((c) => stem(c).hash),
    })}`;
    const job = () => busJob(plan, busPlan, (c) => stem(c).file);
    buses.push([bus.id, await runStem(deps, plan, hash, job, "pcm_f32le")]);
  }
  return buses;
}

async function masterStem(plan: MixPlan, deps: MixDeps, buses: Stem[]): Promise<Stem> {
  const busFiles = buses.map((b) => b.file);
  const hash = `master-${sha({
    fps: plan.fps,
    frames: plan.durationInFrames,
    master: plan.master,
    buses: buses.map((b) => b.hash),
  })}`;
  const job = async () => {
    const measure = loudnessMeasureJob(plan, busFiles);
    if (!measure) return masterJob(plan, busFiles, null);
    const lines: string[] = [];
    await deps.ffmpeg.run(measureArgs(measure), {
      signal: deps.signal,
      onLog: (line) => lines.push(line),
    });
    return masterJob(plan, busFiles, parseLoudnorm(lines.join("\n")));
  };
  const makeup = plan.master.comp.on ? plan.master.comp.makeup : 0;
  return runStem(deps, plan, hash, job, "pcm_s16le", makeup);
}

const levelsOf = (stems: Iterable<[string, Stem]>) =>
  Object.fromEntries([...stems].map(([id, s]) => [id, s.levels]));

async function renderMixStems(plan: MixPlan, deps: MixDeps): Promise<MixResult> {
  if (!hasSound(plan))
    return {
      key: plan.key,
      master: null,
      levels: { channels: {}, buses: {}, master: null },
      reduction: { channels: {}, master: null },
      loudness: null,
    };
  const progress = deps.onProgress ?? (() => undefined);
  const steps = plan.channels.length + plan.buses.length + 2;
  let done = 0;
  const tick: Tick = (message) => progress(done++ / steps, message);

  const channels = await channelStems(plan, deps, tick);
  const buses = await busStems(plan, deps, channels, tick);
  tick("Balancing loudness");
  const master = await masterStem(
    plan,
    deps,
    buses.map(([, s]) => s),
  );
  tick("Measuring loudness");
  const loudness = await loudnessOf(deps, plan, master);
  await remember(deps.projectDir, [
    ...[...channels.values()].map((s) => s.hash),
    ...buses.map(([, s]) => s.hash),
    master.hash,
  ]);
  progress(1, "Sound mixed");
  return {
    key: plan.key,
    master: { src: toProjectRef(deps.projectDir, master.file), hash: master.hash },
    levels: { channels: levelsOf(channels), buses: levelsOf(buses), master: master.levels },
    reduction: {
      channels: Object.fromEntries(
        [...channels].flatMap(([id, s]) => (s.reduction ? [[id, s.reduction]] : [])),
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
  await atomicWrite(indexFile, JSON.stringify(next));
  const keep = new Set(next.flat());
  const cutoff = Date.now() - UNUSED_STEM_MS;
  for (const file of await readdir(dir).catch(() => [] as string[])) {
    if (file === "recent.json") continue;
    const hash = file.split(".")[0];
    if (keep.has(hash)) continue;
    const full = path.join(dir, file);
    const info = await stat(full).catch(() => null);
    if (info && info.mtimeMs < cutoff) await rm(full, { force: true });
  }
}
