import { MIN_DB, SILENCE_DB } from "@/shared/mix/types";

const WHOLE_MS_FROM = 100;
const KILOHERTZ_FROM = 1000;
const WHOLE_KILOHERTZ_FROM = 10_000;
const CENTRE_PAN = 0.01;

export const formatSignedDb = (v: number): string => {
  const text = Math.abs(v).toFixed(1);
  return Number(text) === 0 ? text : `${v > 0 ? "+" : "−"}${text}`;
};

const dbAbove = (floor: number, v: number): string =>
  Number.isNaN(v) || v <= floor ? "−∞" : formatSignedDb(v);

export const formatDb = (v: number): string => dbAbove(MIN_DB, v);

export const formatLevelDb = (v: number): string => dbAbove(SILENCE_DB, v);

export const formatHz = (v: number): string =>
  v >= KILOHERTZ_FROM
    ? `${(v / KILOHERTZ_FROM).toFixed(v >= WHOLE_KILOHERTZ_FROM ? 0 : 1)}k`
    : `${Math.round(v)}`;

export const formatMs = (v: number): string =>
  v >= WHOLE_MS_FROM ? `${Math.round(v)}` : v.toFixed(1);

export function formatPan(v: number): string {
  if (Math.abs(v) < CENTRE_PAN) return "C";
  return v < 0 ? `L${Math.round(-v * 100)}` : `R${Math.round(v * 100)}`;
}
