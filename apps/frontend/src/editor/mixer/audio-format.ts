import { MINUS, signedText } from "@opsis/video/common/format";
import { mixerBook } from "@/store/catalog";

const WHOLE_MS_FROM = 100;
const KILOHERTZ_FROM = 1000;
const WHOLE_KILOHERTZ_FROM = 10_000;
const CENTRE_PAN = 0.01;

const MINUS_INFINITY = `${MINUS}∞`;

export const formatSignedDb = (v: number): string => signedText(v, { digits: 1, plus: true });

export const formatReading = (v: number, floor = -Infinity, plus = false): string =>
  Number.isNaN(v) || v <= floor ? MINUS_INFINITY : signedText(v, { digits: 1, plus });

export const formatDb = (v: number): string => formatReading(v, mixerBook().minDb, true);

export const formatLevelDb = (v: number): string => formatReading(v, mixerBook().silenceDb, true);

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
