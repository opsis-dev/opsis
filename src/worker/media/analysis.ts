import { existsSync } from "node:fs";
import { mkdir, readFile, rename, rm } from "node:fs/promises";
import path from "node:path";
import type { z } from "zod";
import { fileIdentity } from "../../engine/dir";
import { versionedHash } from "../../engine/hash";
import { atomicWrite, readJson } from "../../engine/json";
import { isInside, projectPaths, toProjectRef } from "../../engine/project-paths";
import {
  filmstripSchema,
  waveformSchema,
  type Filmstrip,
  type Waveform,
  type WaveformLevel,
} from "../../shared/analysis";
import { EPSILON, evenPx, round3 } from "../../shared/math";
import type { MediaDeps } from "./ffmpeg";

const ANALYSIS_VERSION = "demoflow-analysis/1";
const DECODE_RATE = 8000;
const LEVEL_RATES = [200, 50, 10];
const MAX_TILES = 100;
const MIN_TILE_MS = 500;
const TILE_HEIGHT = 72;

const ONSETS_PER_SECOND = 100;
const MIN_BEAT_FRAMES = 400;
const MIN_LAG = 33;
const MAX_LAG = 100;
const PRIOR_BPM = 120;
const PRIOR_WIDTH_OCTAVES = 0.9;
const PRIOR_FLOOR = 0.85;
const PRIOR_WEIGHT = 0.15;
const MIN_PERIODICITY = 1.8;
const PEAK_SEARCH_FRAMES = 3;
const FLOAT_BYTES = Float32Array.BYTES_PER_ELEMENT;

async function idFor(file: string): Promise<string> {
  return versionedHash(ANALYSIS_VERSION, { file, identity: await fileIdentity(file) });
}

function filmstripDir(projectDir: string, file: string): string {
  const paths = projectPaths(projectDir);
  const insideNodeFolder =
    isInside(paths.cache, file) &&
    path.dirname(file) !== paths.cache &&
    !isInside(paths.analysis, file) &&
    !isInside(paths.mix, file);
  return insideNodeFolder ? path.dirname(file) : paths.analysis;
}

type Analysis<T> = {
  dir: string;
  name: string;
  tempExt: string;
  schema: z.ZodType<T>;
  ready?: (id: string) => boolean;
  make: (id: string, temp: string) => Promise<T>;
};

async function cachedAnalysis<T>(file: string, analysis: Analysis<T>): Promise<T> {
  const { dir, name } = analysis;
  const id = await idFor(file);
  const json = path.join(dir, `${id}.${name}.json`);
  const cached = (analysis.ready?.(id) ?? true) ? await readJson(json, analysis.schema) : null;
  if (cached) return cached;
  await mkdir(dir, { recursive: true });
  const temp = path.join(dir, `${id}.${name}.${process.pid}.partial.${analysis.tempExt}`);
  try {
    const value = await analysis.make(id, temp);
    await atomicWrite(json, JSON.stringify(value));
    return value;
  } finally {
    await rm(temp, { force: true });
  }
}

const floatSamples = (buf: Buffer): Float32Array =>
  new Float32Array(
    buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength),
    0,
    Math.floor(buf.byteLength / FLOAT_BYTES),
  );

export function waveformLevels(samples: Float32Array, rate: number): WaveformLevel[] {
  return LEVEL_RATES.map((r) => {
    const per = rate / r;
    const count = Math.ceil(samples.length / per);
    const min: number[] = [];
    const max: number[] = [];
    for (let i = 0; i < count; i++) {
      const a = Math.floor(i * per);
      const b = Math.min(samples.length, Math.floor((i + 1) * per));
      let lo = 0;
      let hi = 0;
      for (let k = a; k < b; k++) {
        const v = samples[k];
        if (v < lo) lo = v;
        if (v > hi) hi = v;
      }
      min.push(round3(lo));
      max.push(round3(hi));
    }
    return { rate: r, min, max };
  });
}

function onsetEnvelope(samples: Float32Array, rate: number): Float32Array {
  const hop = Math.round(rate / ONSETS_PER_SECOND);
  const frames = Math.floor(samples.length / hop);
  const energy = new Float32Array(frames);
  for (let f = 0; f < frames; f++) {
    let sum = 0;
    for (let k = f * hop; k < (f + 1) * hop; k++) sum += samples[k] * samples[k];
    energy[f] = Math.log1p(100 * Math.sqrt(sum / hop));
  }
  const onset = new Float32Array(frames);
  for (let f = 1; f < frames; f++) onset[f] = Math.max(0, energy[f] - energy[f - 1]);
  return onset;
}

const tempoPrior = (lag: number) => {
  const bpm = (60 * ONSETS_PER_SECOND) / lag;
  const octaves = Math.log2(bpm / PRIOR_BPM) / PRIOR_WIDTH_OCTAVES;
  return PRIOR_FLOOR + PRIOR_WEIGHT * Math.exp(-0.5 * octaves ** 2);
};

function beatPeriod(onset: Float32Array): number | null {
  const correlations: number[] = [];
  let bestLag = 0;
  let best = 0;
  for (let lag = MIN_LAG; lag <= MAX_LAG; lag++) {
    let ac = 0;
    for (let f = lag; f < onset.length; f++) ac += onset[f] * onset[f - lag];
    correlations.push(ac);
    const score = ac * tempoPrior(lag);
    if (score > best) {
      best = score;
      bestLag = lag;
    }
  }
  const median = correlations.toSorted((a, b) => a - b)[Math.floor(correlations.length / 2)];
  const strongest = correlations[bestLag - MIN_LAG];
  if (!bestLag || median <= 0 || strongest < median * MIN_PERIODICITY) return null;
  return bestLag;
}

function beatPhase(onset: Float32Array, period: number): number {
  let phase = 0;
  let phaseScore = -1;
  for (let p = 0; p < period; p++) {
    let score = 0;
    for (let f = p; f < onset.length; f += period) score += onset[f];
    if (score > phaseScore) {
      phaseScore = score;
      phase = p;
    }
  }
  return phase;
}

export function detectBeats(
  samples: Float32Array,
  rate: number,
): { beats: number[]; bpm: number | null } {
  const none = { beats: [], bpm: null };
  const onset = onsetEnvelope(samples, rate);
  if (onset.length < MIN_BEAT_FRAMES) return none;
  const mean = onset.reduce((a, b) => a + b, 0) / onset.length;
  if (mean <= EPSILON) return none;
  const period = beatPeriod(onset);
  if (period === null) return none;
  const beats: number[] = [];
  for (let f = beatPhase(onset, period); f < onset.length; f += period) {
    let peak = f;
    const last = Math.min(onset.length - 1, f + PEAK_SEARCH_FRAMES);
    for (let k = Math.max(0, f - PEAK_SEARCH_FRAMES); k <= last; k++)
      if (onset[k] > onset[peak]) peak = k;
    beats.push(round3(peak / ONSETS_PER_SECOND));
  }
  return { beats, bpm: Math.round((60 * ONSETS_PER_SECOND) / period) };
}

export async function analyzeWaveform(src: string, deps: MediaDeps): Promise<Waveform> {
  const file = deps.resolve(src);
  const waveform = await cachedAnalysis(file, {
    dir: projectPaths(deps.projectDir).analysis,
    name: "waveform",
    tempExt: "raw",
    schema: waveformSchema.omit({ src: true }),
    make: async (_id, raw) => {
      await deps.ffmpeg.run(
        [
          "-i",
          file,
          "-vn",
          "-ac",
          "1",
          "-ar",
          String(DECODE_RATE),
          "-f",
          "f32le",
          "-c:a",
          "pcm_f32le",
          raw,
        ],
        { signal: deps.signal },
      );
      const samples = floatSamples(await readFile(raw));
      const { beats, bpm } = detectBeats(samples, DECODE_RATE);
      return {
        durationMs: Math.round((samples.length / DECODE_RATE) * 1000),
        levels: waveformLevels(samples, DECODE_RATE),
        beats,
        bpm,
      };
    },
  });
  return { ...waveform, src };
}

export async function analyzeFilmstrip(src: string, deps: MediaDeps): Promise<Filmstrip> {
  const file = deps.resolve(src);
  const dir = filmstripDir(deps.projectDir, file);
  const imageOf = (id: string) => path.join(dir, `${id}.filmstrip.jpg`);
  const strip = await cachedAnalysis(file, {
    dir,
    name: "filmstrip",
    tempExt: "jpg",
    schema: filmstripSchema.omit({ src: true }),
    ready: (id) => existsSync(imageOf(id)),
    make: async (id, tmp) => {
      const probe = await deps.ffmpeg.probe(file);
      if (!probe.video) throw new Error("This file has no picture.");
      const durationMs = Math.max(1, probe.durationMs);
      const intervalMs = Math.max(MIN_TILE_MS, Math.ceil(durationMs / MAX_TILES));
      const count = Math.max(1, Math.ceil(durationMs / intervalMs));
      const tileWidth = Math.max(2, evenPx((probe.video.width / probe.video.height) * TILE_HEIGHT));
      await deps.ffmpeg.run(
        [
          "-i",
          file,
          "-an",
          "-vf",
          `fps=fps=${1000 / intervalMs}:start_time=0,scale=${tileWidth}:${TILE_HEIGHT},tile=${count}x1`,
          "-frames:v",
          "1",
          "-q:v",
          "5",
          tmp,
        ],
        { signal: deps.signal },
      );
      const image = imageOf(id);
      await rename(tmp, image);
      return {
        image: toProjectRef(deps.projectDir, image),
        count,
        intervalMs,
        tileWidth,
        tileHeight: TILE_HEIGHT,
        durationMs,
      };
    },
  });
  return { ...strip, src };
}
