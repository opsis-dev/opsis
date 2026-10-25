import { round1, round2, round3, roundTo } from "#core/math";

const UNITS = ["B", "KB", "MB", "GB"];

export function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "";
  let value = n;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit++;
  }
  return unit === 0 ? `${Math.round(value)} B` : `${value.toFixed(1)} ${UNITS[unit]}`;
}

export const MINUS = "−";

export const signOf = (value: number, plus = false): string =>
  value < 0 ? MINUS : plus && value > 0 ? "+" : "";

export function signedText(
  value: number,
  { digits, plus = false }: { digits?: number; plus?: boolean } = {},
): string {
  const magnitude = Math.abs(value);
  const text = digits === undefined ? String(magnitude) : magnitude.toFixed(digits);
  return Number(text) === 0 ? text : `${signOf(value, plus)}${text}`;
}

const SECONDS_PER_MINUTE = 60;
const ONE_DECIMAL_FROM_SECONDS = 10;

const SECONDS_PER_HOUR = 3600;

function minutesAndSeconds(seconds: number): string {
  const h = Math.floor(seconds / SECONDS_PER_HOUR);
  const m = Math.floor((seconds - h * SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  const rest = round3(seconds - h * SECONDS_PER_HOUR - m * SECONDS_PER_MINUTE);
  if (!h) return rest ? `${m} min ${rest} s` : `${m} min`;
  return [`${h} h`, m ? `${m} min` : "", rest ? `${rest} s` : ""].filter(Boolean).join(" ");
}

export function formatDuration(ms: number): string {
  if (!Number.isFinite(ms)) return "";
  const s = ms / 1000;
  if (s >= SECONDS_PER_MINUTE) return minutesAndSeconds(Math.round(s));
  return `${s >= ONE_DECIMAL_FROM_SECONDS ? round1(s) : round2(s)} s`;
}

export function formatPreciseDuration(ms: number): string {
  if (!Number.isFinite(ms)) return "";
  const s = round3(ms / 1000);
  return s >= SECONDS_PER_MINUTE ? minutesAndSeconds(s) : `${s} s`;
}

export const pascalCase = (key: string): string =>
  key.replace(/(^|[-_])([a-z0-9])/g, (_match, _sep, c: string) => c.toUpperCase());

export function humanize(key: string): string {
  const text = key
    .replace(/[_-]+/g, " ")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .trim()
    .split(/\s+/)
    .map((word) => (/^[A-Z0-9]{2,}$/.test(word) ? word : word.toLowerCase()))
    .join(" ");
  return text.charAt(0).toUpperCase() + text.slice(1);
}

const WHEN_FORMAT = new Intl.DateTimeFormat("en-GB", { dateStyle: "medium", timeStyle: "short" });

export function formatWhen(time: string): string {
  const date = new Date(time);
  return Number.isNaN(date.getTime()) ? time : WHEN_FORMAT.format(date);
}

export function displayAddress(url: string, opts: { keepQuery?: boolean } = {}): string {
  const text = url.trim();
  const scheme = /^https?:\/\//i;
  const parsed = scheme.test(text) && URL.canParse(text) ? new URL(text) : null;
  const address = parsed ? `${parsed.host}${parsed.pathname}` : text.replace(scheme, "");
  const query = parsed && opts.keepQuery ? parsed.search : "";
  return `${address.replace(/^www\./, "").replace(/\/$/, "")}${query}`;
}

const RELATIVE = new Intl.RelativeTimeFormat("en-GB", { numeric: "auto" });

const RELATIVE_STEPS: Array<[Intl.RelativeTimeFormatUnit, number]> = [
  ["year", 31_536_000],
  ["month", 2_592_000],
  ["week", 604_800],
  ["day", 86_400],
  ["hour", 3_600],
  ["minute", 60],
];

export function formatTimeAgo(at: number, now = Date.now()): string {
  const seconds = (at - now) / 1000;
  for (const [unit, size] of RELATIVE_STEPS)
    if (Math.abs(seconds) >= size) return RELATIVE.format(Math.round(seconds / size), unit);
  return "just now";
}

export const plural = (n: number, word: string, many = `${word}s`): string =>
  `${n} ${n === 1 ? word : many}`;

export const singleQuoted = (text: string): string => `'${text.replaceAll("'", "'\\''")}'`;

const PERCENT_DIGITS = 6;

export const percent = (fraction: number): string => `${roundTo(fraction * 100, PERCENT_DIGITS)}%`;

export const wholePercent = (fraction: number): number => Math.round(fraction * 100);

export const percentText = (fraction: number, { max = Infinity }: { max?: number } = {}): string =>
  `${Math.min(max, wholePercent(fraction))}%`;
