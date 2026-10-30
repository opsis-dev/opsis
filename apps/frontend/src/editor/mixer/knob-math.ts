import { clamp, clamp01, normalize, type StepRange } from "@opsis/video/common/math";
import type { NumberRange } from "@/generated/bindings";

export const SWEEP_DEG = 270;
export const START_DEG = -SWEEP_DEG / 2;

export type KnobRange = StepRange & { log?: boolean };

export function rangeProps(range: NumberRange) {
  return {
    min: range.min,
    max: range.max,
    reset: range.default,
  };
}

export function roundToStep(v: number, step: number): number {
  const decimals = Math.max(0, -Math.floor(Math.log10(step)));
  return Number((Math.round(v / step) * step).toFixed(decimals));
}

export function valueToFraction({ min, max, log }: KnobRange, v: number): number {
  return log ? clamp01(Math.log(v / min) / Math.log(max / min)) : normalize(v, min, max);
}

export function fractionToValue(range: KnobRange, fraction: number): number {
  const { min, max, log, step } = range;
  const c = clamp01(fraction);
  return roundToStep(log ? min * (max / min) ** c : min + c * (max - min), step);
}

export function snapValue(range: KnobRange, v: number): number {
  return roundToStep(clamp(v, range.min, range.max), range.step);
}

export function angleOf(fraction: number): number {
  return START_DEG + fraction * SWEEP_DEG;
}
