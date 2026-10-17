import type { MixPlan } from "./mix/plan";
import type { MixResult } from "./mix/result";

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
  | { kind: "error"; error: string; engineMissing?: true };
