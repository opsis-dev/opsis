import { randomUUID } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, readFile, rename, rm } from "node:fs/promises";
import path from "node:path";
import { z } from "zod";
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
import { channelIdentity, hasSound, type MixPlan } from "../../shared/mix/plan";
import {
  loudnessSchema,
  stemLevelsSchema,
  type Loudness,
  type MixResult,
  type StemLevels,
} from "../../shared/mix/result";
import { ToolError, type MediaDeps } from "../media/ffmpeg";
import { shareInFlight } from "../../shared/memo";
import { frameLevels, parseWav } from "./wav";
import { SILENCE_DB } from "../../shared/mix/types";
import { CancelledError, errorMessage, isCancelled } from "../../shared/errors";
import { EPSILON, round1 } from "../../shared/math";
import { fileIdentity, pruneDir } from "../../engine/dir";
import { versionedHash } from "../../engine/hash";
import { atomicWrite, readJson } from "../../engine/json";
import { projectPaths, toProjectRef } from "../../engine/project-paths";

const MIX_VERSION = "demoflow-mix/1";
const KEEP_PLANS = 12;
const UNUSED_STEM_MS = 10 * 60_000;
const LOUDNESS_FLOOR_DB = -70;
const QUIET_PRE_DB = -100;
const RECENT_FILE = "recent.json";

type MixDeps = MediaDeps & { onProgress?: (progress: number, message: string) => void };

const mixDir = (projectDir: string) => projectPaths(projectDir).mix;

const sha = (value: unknown) => versionedHash(MIX_VERSION, value);

const identity = (file: string): Promise<string> => fileIdentity(file).catch(() => "missing");

type Stem = { hash: string; file: string; levels: StemLevels; reduction: number[] | null };

const stemMetaSchema = z.object({
  levels: stemLevelsSchema,
  reduction: z.array(z.number()).nullable(),
});

const recentSchema = z.array(z.array(z.string()));

const stemRenders = shareInFlight<Stem>();

const runStem = (...args: Parameters<typeof renderStem>): Promise<Stem> =>
  stemRenders(path.join(mixDir(args[0].projectDir), `${args[2]}.wav`), () => renderStem(...args));

async function renderStem(
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
  const cached = existsSync(file) ? await readJson(meta, stemMetaSchema) : null;
  if (cached) return { hash, file, ...cached };
  await mkdir(dir, { recursive: true });
  const job = await makeJob();
  const unique = randomUUID();
  const tmp = `${file}.${unique}.partial.wav`;
  const taps = job.taps
    ? { pre: `${file}.${unique}.pre.wav`, post: `${file}.${unique}.post.wav` }
    : undefined;
  const levelsOf = async (wav: string) =>
    frameLevels(parseWav(await readFile(wav)), plan.fps, plan.durationInFrames);
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
        v <= QUIET_PRE_DB ? 0 : Math.max(0, round1(v - (post.rms[i] - makeup))),
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
  const cached = await readJson(file, loudnessSchema);
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
    while (k + 1 < samples.length && samples[k + 1].t <= at + EPSILON) k++;
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
  if (isCancelled(error)) return new CancelledError();
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
    const hash = `ch-${sha({ channel: channelIdentity(plan, channel), ids })}`;
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

const levelsById = (stems: Iterable<[string, Stem]>) =>
  Object.fromEntries([...stems].map(([id, s]) => [id, s.levels]));

const emptyMixResult = (plan: MixPlan): MixResult => ({
  key: plan.key,
  master: null,
  levels: { channels: {}, buses: {}, master: null },
  reduction: { channels: {}, master: null },
  loudness: null,
});

const MASTER_STEPS = ["Balancing loudness", "Measuring loudness"] as const;

async function renderMixStems(plan: MixPlan, deps: MixDeps): Promise<MixResult> {
  if (!hasSound(plan)) return emptyMixResult(plan);
  const progress = deps.onProgress ?? (() => undefined);
  const steps = plan.channels.length + plan.buses.length + MASTER_STEPS.length;
  let done = 0;
  const tick: Tick = (message) => progress(done++ / steps, message);

  const channels = await channelStems(plan, deps, tick);
  const buses = await busStems(plan, deps, channels, tick);
  tick(MASTER_STEPS[0]);
  const master = await masterStem(
    plan,
    deps,
    buses.map(([, s]) => s),
  );
  tick(MASTER_STEPS[1]);
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
    levels: { channels: levelsById(channels), buses: levelsById(buses), master: master.levels },
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
  const indexFile = path.join(dir, RECENT_FILE);
  const recent = (await readJson(indexFile, recentSchema)) ?? [];
  const next = [hashes, ...recent.filter((h) => h.join() !== hashes.join())].slice(0, KEEP_PLANS);
  await atomicWrite(indexFile, JSON.stringify(next));
  const keep = new Set(next.flat());
  await pruneDir(dir, {
    keep: (name) => name === RECENT_FILE || keep.has(name.split(".")[0]),
    unusedForMs: UNUSED_STEM_MS,
  });
}
