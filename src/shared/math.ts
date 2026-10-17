export const clamp = (value: number, lo: number, hi: number): number =>
  Math.min(hi, Math.max(lo, value));

export const clamp01 = (value: number): number => clamp(value, 0, 1);

export const round1 = (value: number): number => Math.round(value * 10) / 10;

export const round2 = (value: number): number => Math.round(value * 100) / 100;

export const round3 = (value: number): number => Math.round(value * 1000) / 1000;

export const numberOr = (value: unknown, fallback: number): number =>
  typeof value === "number" ? value : fallback;
