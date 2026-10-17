import { chordKeys } from "../../../shared/format";
import type { Rect } from "../../../shared/schema/actions";

export type ClickEvent = {
  label?: string;
  tMs: number;
  tDepartMs?: number;
  tDownMs?: number;
  typeEndMs?: number;
  x: number;
  y: number;
  rect?: Rect;
  cluster?: string;
  zoom?: boolean;
  zoomScale?: number;
  tUpMs?: number;
  path?: Array<{ x: number; y: number }>;
  drop?: Rect;
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

const KEY_GLYPHS = new Map([
  ["alt", "\u2325"],
  ["option", "\u2325"],
  ["meta", "\u2318"],
  ["cmd", "\u2318"],
  ["command", "\u2318"],
  ["control", "\u2303"],
  ["ctrl", "\u2303"],
  ["shift", "\u21e7"],
  ["enter", "\u23ce"],
  ["return", "\u23ce"],
  ["tab", "\u21e5"],
  ["escape", "esc"],
  ["esc", "esc"],
  ["backspace", "\u232b"],
  ["delete", "\u2326"],
  ["arrowup", "\u2191"],
  ["arrowdown", "\u2193"],
  ["arrowleft", "\u2190"],
  ["arrowright", "\u2192"],
  ["space", "space"],
]);

export const chordGlyphs = (chord: string): string =>
  chordKeys(chord)
    .map((part) => {
      const key = part.trim();
      return KEY_GLYPHS.get(key.toLowerCase()) ?? key.toUpperCase();
    })
    .join("");

export type ClickLog = {
  name: string;
  viewport: { width: number; height: number };
  cursorTrack?: CursorSample[];
  durationMs: number;
  clicks: ClickEvent[];
  keys?: KeyEvent[];
};

export const DESIGN_WIDTH = 1920;

export function defaultTargetRect(point: { x: number; y: number }, viewportWidth: number): Rect {
  const k = viewportWidth > 0 ? viewportWidth / DESIGN_WIDTH : 1;
  return { x: point.x - 60 * k, y: point.y - 24 * k, w: 120 * k, h: 48 * k };
}

export const LEAD_FALLBACK_S = 0.75;
export const HOLD_MIN_S = 1.3;
export const TYPE_TAIL_S = 0.4;
export const CAMERA_LEAD_S = 0.035;
export const BASE_BEAT_S = 0.6;

export const CURSOR_SCALE_EXP = 0.58;
export const RIPPLE_S = 0.35;
export const CLICK_SQUASH = 0.9;
export const CLICK_SQUASH_S = 0.12;
export const CURSOR_FADE_S = 0.18;
