import type { MixPlan } from "./plan";

export type StemLevels = { fps: number; peak: number[]; rms: number[] };

export type Loudness = {
  fps: number;
  momentary: number[];
  shortTerm: number[];
  integrated: number | null;
  range: number | null;
  truePeak: number | null;
};

export type MixResult = {
  key: string;
  master: { src: string; hash: string } | null;
  levels: {
    channels: Record<string, StemLevels>;
    buses: Record<string, StemLevels>;
    master: StemLevels | null;
  };
  reduction: { channels: Record<string, number[]>; master: number[] | null };
  loudness: Loudness | null;
};

export type WaveformLevel = { rate: number; min: number[]; max: number[] };

export type Waveform = {
  src: string;
  durationMs: number;
  levels: WaveformLevel[];
  beats: number[];
  bpm: number | null;
};

export type Filmstrip = {
  src: string;
  image: string;
  count: number;
  intervalMs: number;
  tileWidth: number;
  tileHeight: number;
  durationMs: number;
};

export type AnalysisRequest =
  | { kind: "waveform"; projectDir: string; src: string }
  | { kind: "filmstrip"; projectDir: string; src: string }
  | { kind: "mix"; projectDir: string; plan: MixPlan };

export type AnalysisResult =
  | { kind: "waveform"; waveform: Waveform }
  | { kind: "filmstrip"; filmstrip: Filmstrip }
  | { kind: "mix"; mix: MixResult }
  | { kind: "skipped" }
  | { kind: "error"; error: string };
