import { clamp, clamp01 } from "@/shared/math";
import type { NumberRange } from "@/shared/mix/types";

export const SWEEP_DEG = 270;
export const START_DEG = -SWEEP_DEG / 2;

export type KnobRange = { min: number; max: number; step: number; log?: boolean };

export const rangeProps = (range: NumberRange) => ({
  min: range.min,
  max: range.max,
  reset: range.default,
});

export function roundToStep(v: number, step: number): number {
  const decimals = Math.max(0, -Math.floor(Math.log10(step)));
  return Number((Math.round(v / step) * step).toFixed(decimals));
}

export const valueToFraction = ({ min, max, log }: KnobRange, v: number): number =>
  log ? Math.log(v / min) / Math.log(max / min) : (v - min) / (max - min);

export function fractionToValue(range: KnobRange, fraction: number): number {
  const { min, max, log, step } = range;
  const c = clamp01(fraction);
  return roundToStep(log ? min * (max / min) ** c : min + c * (max - min), step);
}

export const snapValue = (range: KnobRange, v: number): number =>
  roundToStep(clamp(v, range.min, range.max), range.step);

export const angleOf = (fraction: number): number => START_DEG + fraction * SWEEP_DEG;

export function polar(center: number, radius: number, deg: number): { x: number; y: number } {
  const rad = ((deg - 90) * Math.PI) / 180;
  return { x: center + radius * Math.cos(rad), y: center + radius * Math.sin(rad) };
}

export function arcPath(center: number, radius: number, from: number, to: number): string {
  const a = polar(center, radius, from);
  const b = polar(center, radius, to);
  return `M ${a.x} ${a.y} A ${radius} ${radius} 0 ${to - from > 180 ? 1 : 0} 1 ${b.x} ${b.y}`;
}
