import { MIN_DB } from "@/shared/mix/types";

export const db = (v: number): string =>
  Number.isNaN(v) || v <= MIN_DB ? "−∞" : `${v > 0 ? "+" : ""}${v.toFixed(1)}`;

export const hz = (v: number): string =>
  v >= 1000 ? `${(v / 1000).toFixed(v >= 10_000 ? 0 : 1)}k` : `${Math.round(v)}`;

export const ms = (v: number): string => (v >= 100 ? `${Math.round(v)}` : v.toFixed(1));

export function panLabel(v: number): string {
  if (Math.abs(v) < 0.01) return "C";
  return v < 0 ? `L${Math.round(-v * 100)}` : `R${Math.round(v * 100)}`;
}
