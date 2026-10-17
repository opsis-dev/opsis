import { existsSync } from "node:fs";
import { mkdir, readFile, rename, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Filmstrip, Waveform, WaveformLevel } from "../../shared/analysis";
import type { FfmpegTools } from "./ffmpeg";
import { sha256Hex } from "../../engine/hash";
import { readJson } from "../../engine/json";
import { toProjectRef } from "../../engine/project-paths";
import { round3 } from "../../shared/math";

export const ANALYSIS_VERSION = "demoflow-analysis/1";
const DECODE_RATE = 8000;
const LEVEL_RATES = [200, 50, 10];
const MAX_TILES = 100;
const TILE_HEIGHT = 72;

export type AnalysisDeps = {
  projectDir: string;
  resolve: (ref: string) => string;
  ffmpeg: FfmpegTools;
  signal?: AbortSignal;
};

async function idFor(file: string): Promise<string> {
  const s = await stat(file);
  return sha256Hex(`${ANALYSIS_VERSION}\0${file}\0${s.size}\0${Math.round(s.mtimeMs)}`, 24);
}

function outDir(projectDir: string, file: string): string {
  const cache = path.join(projectDir, ".demoflow", "cache");
  const rel = path.relative(cache, file);
  const parts = rel.split(path.sep);
  if (
    !rel.startsWith("..") &&
    !path.isAbsolute(rel) &&
    parts.length >= 2 &&
    parts[0] !== "analysis" &&
    parts[0] !== "mix"
  )
    return path.dirname(file);
  return path.join(cache, "analysis");
}

export function waveformLevels(
  samples: Float32Array,
  rate: number,
  rates = LEVEL_RATES,
): WaveformLevel[] {
  return rates.map((r) => {
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

export function detectBeats(
  samples: Float32Array,
  rate: number,
): { beats: number[]; bpm: number | null } {
  const hop = Math.round(rate / 100);
  const frames = Math.floor(samples.length / hop);
  if (frames < 400) return { beats: [], bpm: null };
  const energy = new Float32Array(frames);
  for (let f = 0; f < frames; f++) {
    let sum = 0;
    for (let k = f * hop; k < (f + 1) * hop; k++) sum += samples[k] * samples[k];
    energy[f] = Math.log1p(100 * Math.sqrt(sum / hop));
  }
  const onset = new Float32Array(frames);
  for (let f = 1; f < frames; f++) onset[f] = Math.max(0, energy[f] - energy[f - 1]);
  const mean = onset.reduce((a, b) => a + b, 0) / frames;
  if (mean <= 1e-6) return { beats: [], bpm: null };
  const acs: number[] = [];
  let bestLag = 0;
  let best = 0;
  for (let lag = 33; lag <= 100; lag++) {
    let ac = 0;
    for (let f = lag; f < frames; f++) ac += onset[f] * onset[f - lag];
    acs.push(ac);
    const bpm = 6000 / lag;
    const score = ac * (0.85 + 0.15 * Math.exp(-0.5 * (Math.log2(bpm / 120) / 0.9) ** 2));
    if (score > best) {
      best = score;
      bestLag = lag;
    }
  }
  const sorted = acs.toSorted((a, b) => a - b);
  const median = sorted[Math.floor(sorted.length / 2)];
  const strongest = acs[bestLag - 33];
  if (!bestLag || median <= 0 || strongest < median * 1.8) return { beats: [], bpm: null };
  let phase = 0;
  let phaseScore = -1;
  for (let p = 0; p < bestLag; p++) {
    let sc = 0;
    for (let f = p; f < frames; f += bestLag) sc += onset[f];
    if (sc > phaseScore) {
      phaseScore = sc;
      phase = p;
    }
  }
  const beats: number[] = [];
  for (let f = phase; f < frames; f += bestLag) {
    let peak = f;
    for (let k = Math.max(0, f - 3); k <= Math.min(frames - 1, f + 3); k++)
      if (onset[k] > onset[peak]) peak = k;
    beats.push(round3(peak / 100));
  }
  return { beats, bpm: Math.round(6000 / bestLag) };
}

export async function analyzeWaveform(src: string, deps: AnalysisDeps): Promise<Waveform> {
  const file = deps.resolve(src);
  const id = await idFor(file);
  const dir = path.join(deps.projectDir, ".demoflow", "cache", "analysis");
  const json = path.join(dir, `${id}.waveform.json`);
  const cached = await readJson<Waveform>(json);
  if (cached) return { ...cached, src };
  await mkdir(dir, { recursive: true });
  const raw = path.join(dir, `${id}.${process.pid}.raw`);
  try {
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
    const buf = await readFile(raw);
    const samples = new Float32Array(buf.buffer, buf.byteOffset, Math.floor(buf.byteLength / 4));
    const { beats, bpm } = detectBeats(samples, DECODE_RATE);
    const waveform: Waveform = {
      src,
      durationMs: Math.round((samples.length / DECODE_RATE) * 1000),
      levels: waveformLevels(samples, DECODE_RATE),
      beats,
      bpm,
    };
    await writeFile(json, JSON.stringify(waveform));
    return waveform;
  } finally {
    await rm(raw, { force: true });
  }
}

export async function analyzeFilmstrip(src: string, deps: AnalysisDeps): Promise<Filmstrip> {
  const file = deps.resolve(src);
  const id = await idFor(file);
  const dir = outDir(deps.projectDir, file);
  const base =
    dir === path.dirname(file) ? `${path.parse(file).name}.filmstrip` : `${id}.filmstrip`;
  const json = path.join(dir, `${base}.json`);
  const image = path.join(dir, `${base}.jpg`);
  const cached = await readJson<Filmstrip & { id: string }>(json);
  if (cached && cached.id === id && existsSync(image)) {
    const { id: _id, ...rest } = cached;
    return { ...rest, src };
  }
  await mkdir(dir, { recursive: true });
  const probe = await deps.ffmpeg.probe(file);
  if (!probe.video) throw new Error("This file has no picture.");
  const durationMs = Math.max(1, probe.durationMs);
  const intervalMs = Math.max(500, Math.ceil(durationMs / MAX_TILES));
  const count = Math.max(1, Math.ceil(durationMs / intervalMs));
  const tileHeight = TILE_HEIGHT;
  const tileWidth = Math.max(
    2,
    Math.round(((probe.video.width / probe.video.height) * tileHeight) / 2) * 2,
  );
  const tmp = `${image}.${process.pid}.partial.jpg`;
  try {
    await deps.ffmpeg.run(
      [
        "-i",
        file,
        "-an",
        "-vf",
        `fps=fps=${1000 / intervalMs}:start_time=0,scale=${tileWidth}:${tileHeight},tile=${count}x1`,
        "-frames:v",
        "1",
        "-q:v",
        "5",
        tmp,
      ],
      { signal: deps.signal },
    );
    await rename(tmp, image);
  } finally {
    await rm(tmp, { force: true });
  }
  const strip: Filmstrip = {
    src,
    image: toProjectRef(deps.projectDir, image),
    count,
    intervalMs,
    tileWidth,
    tileHeight,
    durationMs,
  };
  await writeFile(json, JSON.stringify({ ...strip, id }));
  return strip;
}
