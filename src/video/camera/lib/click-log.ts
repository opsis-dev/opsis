export type ClickEvent = {
  label?: string;
  tMs: number;
  tDepartMs?: number;
  tDownMs?: number;
  typeEndMs?: number;
  x: number;
  y: number;
  rect?: { x: number; y: number; w: number; h: number };
  cluster?: string;
  zoom?: boolean;
  zoomScale?: number;
  tUpMs?: number;
  path?: Array<{ x: number; y: number }>;
  drop?: { x: number; y: number; w: number; h: number };
};

export type CursorSample = {
  t: number;
  x: number;
  y: number;
};

export type KeyEvent = {
  tMs: number;
  chord: string;
  label?: string;
};

const KEY_GLYPHS: Record<string, string> = {
  alt: "\u2325",
  option: "\u2325",
  meta: "\u2318",
  cmd: "\u2318",
  command: "\u2318",
  control: "\u2303",
  ctrl: "\u2303",
  shift: "\u21e7",
  enter: "\u23ce",
  return: "\u23ce",
  tab: "\u21e5",
  escape: "esc",
  esc: "esc",
  backspace: "\u232b",
  delete: "\u2326",
  arrowup: "\u2191",
  arrowdown: "\u2193",
  arrowleft: "\u2190",
  arrowright: "\u2192",
  space: "space",
};

export const chordGlyphs = (chord: string): string =>
  chord
    .split("+")
    .map((part) => {
      const key = part.trim();
      return KEY_GLYPHS[key.toLowerCase()] ?? key.toUpperCase();
    })
    .join("");

export type ClickLog = {
  name: string;
  viewport: { width: number; height: number };
  cursorTrack?: CursorSample[];
  durationMs: number;
  trimBeforeMs?: number;
  offsetMs?: number;
  backdrop?: string;
  clicks: ClickEvent[];
  keys?: KeyEvent[];
};

export const EMPTY_LOG: ClickLog = {
  name: "",
  viewport: { width: 1920, height: 1080 },
  durationMs: 8000,
  offsetMs: 0,
  clicks: [],
};

export const OUTPUT_WIDTH = 2560;
export const DESIGN_WIDTH = 1920;

export function defaultTargetRect(
  point: { x: number; y: number },
  viewportWidth: number,
): { x: number; y: number; w: number; h: number } {
  const k = viewportWidth > 0 ? viewportWidth / DESIGN_WIDTH : 1;
  return { x: point.x - 60 * k, y: point.y - 24 * k, w: 120 * k, h: 48 * k };
}

export const LEAD_FALLBACK_S = 0.75;
export const HOLD_MIN_S = 1.3;
export const TYPE_TAIL_S = 0.4;
export const CAMERA_LEAD_S = 0.035;
export const END_TAIL_S = 2.5;
export const BASE_BEAT_S = 0.6;

export const CURSOR_SCALE_EXP = 0.58;
export const RIPPLE_S = 0.35;
export const CLICK_SQUASH = 0.9;
export const CLICK_SQUASH_S = 0.12;
export const CURSOR_FADE_S = 0.18;
export const DEFAULT_PLAYBACK_RATE = 1.25;

export function resolvePlaybackRate(raw?: number | string): number {
  const n =
    typeof raw === "number" ? raw : raw != null && raw !== "" ? Number(raw) : DEFAULT_PLAYBACK_RATE;
  if (!Number.isFinite(n) || n <= 0) return DEFAULT_PLAYBACK_RATE;
  return n;
}
export const ESTABLISH_CUSHION_S = 1.0;
export const TRAIL_CEILING_S = 1.45;

export const clusterGapMs = (): number => Math.round((HOLD_MIN_S + TRAIL_CEILING_S) * 1000);

export const DEVICE_SCALE_FACTOR = 2;
export const RECORD_SLOW_MO_MS = 0;
