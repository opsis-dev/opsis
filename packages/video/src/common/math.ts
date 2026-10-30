export const EPSILON = 1e-6;

export function clamp(value: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, value));
}

export type MinMax = { min: number; max: number };

export type StepRange = MinMax & { step: number };

export function clamp01(value: number): number {
  return clamp(value, 0, 1);
}

export function normalize(value: number, min: number, max: number): number {
  return clamp01((value - min) / (max - min));
}

export function lerp(from: number, to: number, t: number): number {
  return from + (to - from) * t;
}

export function smoothstep(value: number): number {
  const t = clamp01(value);
  return t * t * (3 - 2 * t);
}

export function easeOutQuad(value: number): number {
  return 1 - (1 - clamp01(value)) ** 2;
}

export function easeOutCubic(value: number): number {
  return 1 - (1 - clamp01(value)) ** 3;
}

export function roundTo(value: number, digits: number): number {
  const scale = 10 ** digits;
  return Math.round(value * scale) / scale;
}

export function round2(value: number): number {
  return roundTo(value, 2);
}

export function round3(value: number): number {
  return roundTo(value, 3);
}

export function finiteNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

export function numberOr(value: unknown, fallback: number): number {
  return finiteNumber(value) ?? fallback;
}

export function partitionPoint<T>(items: readonly T[], before: (item: T) => boolean): number {
  let lo = 0;
  let hi = items.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (before(items[mid])) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

export function sampleKeyed<K, V>(
  keys: readonly [K, ...K[]],
  x: number,
  at: (key: K) => number,
  valueOf: (key: K) => V,
  mix: (a: K, b: K, t: number) => V,
): V;
export function sampleKeyed<K, V>(
  keys: readonly K[],
  x: number,
  at: (key: K) => number,
  valueOf: (key: K) => V,
  mix: (a: K, b: K, t: number) => V,
): V | undefined;
export function sampleKeyed<K, V>(
  keys: readonly K[],
  x: number,
  at: (key: K) => number,
  valueOf: (key: K) => V,
  mix: (a: K, b: K, t: number) => V,
): V | undefined {
  if (keys.length === 0) return undefined;
  if (x <= at(keys[0])) return valueOf(keys[0]);
  const last = keys[keys.length - 1];
  if (x >= at(last)) return valueOf(last);
  const i = partitionPoint(keys, (k) => at(k) <= x);
  const a = keys[i - 1];
  const b = keys[i];
  return mix(a, b, (x - at(a)) / (at(b) - at(a)));
}
