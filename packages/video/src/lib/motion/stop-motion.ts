import { random } from "@opsis/runtime/random.ts";
export const DEFAULT_STEP = 3;

export function qf(frame: number, step = DEFAULT_STEP): number {
  return Math.floor(frame / step) * step;
}

export function qstep(frame: number, step = DEFAULT_STEP): number {
  return Math.floor(frame / step);
}

export function randomBetween(seed: string, lo: number, hi: number): number {
  return lo + random(seed) * (hi - lo);
}

export function steppedRamp(
  frame: number,
  from: number,
  to: number,
  options?: { ease?: (t: number) => number; step?: number },
): number {
  const f = qf(frame, options?.step ?? DEFAULT_STEP);
  if (f <= from) return 0;
  if (f >= to) return 1;
  const ease = options?.ease ?? ((t: number) => t);
  return ease((f - from) / (to - from));
}
