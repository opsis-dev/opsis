import { MINUS, signedText } from "@opsis/video/common/format";
import { mixerBook } from "@/store/catalog";

const WHOLE_MS_FROM = 100;
const KILOHERTZ_FROM = 1000;
const WHOLE_KILOHERTZ_FROM = 10_000;
const CENTRE_PAN = 0.01;

const MINUS_INFINITY = `${MINUS}∞`;

export function formatSignedDb(v: number): string {
  return signedText(v, { digits: 1, plus: true });
}

export function formatReading(v: number, floor = -Infinity, plus = false): string {
  return Number.isNaN(v) || v <= floor ? MINUS_INFINITY : signedText(v, { digits: 1, plus });
}

export function formatDb(v: number): string {
  return formatReading(v, mixerBook().minDb, true);
}

export function formatLevelDb(v: number): string {
  return formatReading(v, mixerBook().silenceDb, true);
}

export function formatHz(v: number): string {
  return v >= KILOHERTZ_FROM
    ? `${(v / KILOHERTZ_FROM).toFixed(v >= WHOLE_KILOHERTZ_FROM ? 0 : 1)}k`
    : `${Math.round(v)}`;
}

export function formatMs(v: number): string {
  return v >= WHOLE_MS_FROM ? `${Math.round(v)}` : v.toFixed(1);
}

export function formatPan(v: number): string {
  if (Math.abs(v) < CENTRE_PAN) return "C";
  return v < 0 ? `L${Math.round(-v * 100)}` : `R${Math.round(v * 100)}`;
}
