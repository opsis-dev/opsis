import type { MixResult, StemLevels } from "@/shared/mix/result";
import { frameIn } from "@/shared/timeline/time";

const SILENT_POWER = 1e-12;

export type Levels = Pick<StemLevels, "peak" | "rms"> | null | undefined;

export type Reduction = number[] | null | undefined;

export type Level = { peak: number; rms: number; hold: number } | null;

export const sampleAt = (values: number[] | null | undefined, frame: number): number | null =>
  values?.length ? values[frameIn(frame, values.length)] : null;

export const reductionAt = (reduction: Reduction, frame: number): number | null =>
  reduction ? (sampleAt(reduction, frame) ?? 0) : null;

const power = (db: number) => 10 ** (db / 10);

export function share(
  result: MixResult | null,
  levels: Pick<StemLevels, "rms"> | undefined,
  frame: number,
): number {
  const master = sampleAt(result?.levels.master?.rms, frame);
  const channel = sampleAt(levels?.rms, frame);
  if (master === null || channel === null) return 1;
  const pm = power(master);
  return pm <= SILENT_POWER ? 1 : Math.min(1, power(channel) / pm);
}

export function levelAt(levels: Levels, frame: number, holdFrames: number): Level {
  if (!levels?.peak.length) return null;
  const f = frameIn(frame, levels.peak.length);
  const hold = Math.max(...levels.peak.slice(Math.max(0, f - holdFrames), f + 1));
  return { peak: levels.peak[f], rms: levels.rms[f], hold };
}
