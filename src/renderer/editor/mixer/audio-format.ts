import { MIN_DB, SILENCE_DB } from "@/shared/mix/types";

export const signedDb = (v: number): string => {
  const text = Math.abs(v).toFixed(1);
  return Number(text) === 0 ? text : `${v > 0 ? "+" : "−"}${text}`;
};

const dbAbove = (floor: number, v: number): string =>
  Number.isNaN(v) || v <= floor ? "−∞" : signedDb(v);

export const db = (v: number): string => dbAbove(MIN_DB, v);

export const levelDb = (v: number): string => dbAbove(SILENCE_DB, v);

export const hz = (v: number): string =>
  v >= 1000 ? `${(v / 1000).toFixed(v >= 10_000 ? 0 : 1)}k` : `${Math.round(v)}`;

export const ms = (v: number): string => (v >= 100 ? `${Math.round(v)}` : v.toFixed(1));

export function panLabel(v: number): string {
  if (Math.abs(v) < 0.01) return "C";
  return v < 0 ? `L${Math.round(-v * 100)}` : `R${Math.round(v * 100)}`;
}
