import { z } from "zod";
import type { MixPlan } from "#core/mix/plan";
import type { MixResult } from "#core/mix/result";

const numbers = z.array(z.number());

const waveformLevelSchema = z.object({ rate: z.number(), min: numbers, max: numbers });
export type WaveformLevel = z.infer<typeof waveformLevelSchema>;

export const waveformSchema = z.object({
  src: z.string(),
  durationMs: z.number(),
  levels: z.array(waveformLevelSchema),
  beats: numbers,
  bpm: z.number().nullable(),
});
export type Waveform = z.infer<typeof waveformSchema>;

export const filmstripSchema = z.object({
  src: z.string(),
  image: z.string(),
  count: z.number(),
  intervalMs: z.number(),
  tileWidth: z.number(),
  tileHeight: z.number(),
  durationMs: z.number(),
});
export type Filmstrip = z.infer<typeof filmstripSchema>;

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

export type AnalysisKind = AnalysisRequest["kind"];

export type AnalysisResultFor<K extends AnalysisKind> = Extract<
  AnalysisResult,
  { kind: K | "skipped" | "error" }
>;
