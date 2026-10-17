export const clamp = (value: number, lo: number, hi: number): number =>
  Math.min(hi, Math.max(lo, value));

export const clamp01 = (value: number): number => clamp(value, 0, 1);

export const lerp = (from: number, to: number, t: number): number => from + (to - from) * t;

export const round1 = (value: number): number => Math.round(value * 10) / 10;

export const round2 = (value: number): number => Math.round(value * 100) / 100;

export const round3 = (value: number): number => Math.round(value * 1000) / 1000;

export const finiteNumber = (value: unknown): number | undefined =>
  typeof value === "number" && Number.isFinite(value) ? value : undefined;

export const numberOr = (value: unknown, fallback: number): number =>
  finiteNumber(value) ?? fallback;

export const finiteOr = (value: unknown, fallback: number): number => {
  const n = Number(value);
  return Number.isFinite(n) ? n : fallback;
};

export const evenPx = (value: number): number => Math.round(value / 2) * 2;

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
