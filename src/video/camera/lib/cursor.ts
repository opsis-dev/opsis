import {
  CLICK_SQUASH,
  CLICK_SQUASH_S,
  CURSOR_FADE_S,
  RIPPLE_S,
  type ClickEvent,
  type CursorSample,
} from "./click-log";
import { partitionPoint } from "./camera";
import { clamp, lerp } from "../../../shared/math";

type CursorState = {
  x: number;
  y: number;
  squash: number;
  ripple: number | null;
  opacity: number;
};

export function cursorOpacity(clicks: ClickEvent[], tS: number): number {
  let opacity = 1;
  for (const c of clicks) {
    if (c.typeEndMs == null) continue;
    const from = c.tMs / 1000;
    const to = c.typeEndMs / 1000;
    if (tS <= from - CURSOR_FADE_S || tS >= to + CURSOR_FADE_S) continue;
    const fadingOut = clamp((from - tS) / CURSOR_FADE_S, 0, 1);
    const fadingIn = clamp((tS - to) / CURSOR_FADE_S, 0, 1);
    opacity = Math.min(opacity, Math.max(fadingOut, fadingIn));
  }
  return opacity;
}

export function samplePath(track: CursorSample[], tS: number): { x: number; y: number } | null {
  if (track.length === 0) return null;
  const tMs = tS * 1000;
  if (tMs <= track[0].t) return { x: track[0].x, y: track[0].y };
  const last = track[track.length - 1];
  if (tMs >= last.t) return { x: last.x, y: last.y };

  const hi = partitionPoint(track, (s) => s.t <= tMs);
  const a = track[hi - 1];
  const b = track[hi];
  const span = b.t - a.t;
  const p = span < 1e-6 ? 0 : (tMs - a.t) / span;
  return { x: lerp(a.x, b.x, p), y: lerp(a.y, b.y, p) };
}

function squashFor(c: ClickEvent, tS: number): number {
  if (c.tDownMs == null) return 1;
  const since = tS - c.tDownMs / 1000;
  if (since < 0) return 1;
  const held = c.tUpMs != null ? Math.max(0, (c.tUpMs - c.tDownMs) / 1000) : 0;
  const released = since - held;
  if (released <= 0) return CLICK_SQUASH;
  if (released > CLICK_SQUASH_S) return 1;
  return lerp(CLICK_SQUASH, 1, clamp(released / CLICK_SQUASH_S, 0, 1));
}

function rippleFor(c: ClickEvent, tS: number): number | null {
  if (c.tDownMs == null) return null;
  const since = tS - c.tDownMs / 1000;
  return since >= 0 && since <= RIPPLE_S ? clamp(since / RIPPLE_S, 0, 1) : null;
}

export function cursorAt(
  track: CursorSample[],
  clicks: ClickEvent[],
  tS: number,
): CursorState | null {
  const here = samplePath(track, tS);
  if (!here) return null;

  let squash = 1;
  let ripple: number | null = null;
  for (const c of clicks) {
    squash = Math.min(squash, squashFor(c, tS));
    ripple = rippleFor(c, tS) ?? ripple;
  }

  return {
    x: here.x,
    y: here.y,
    squash,
    ripple,
    opacity: cursorOpacity(clicks, tS),
  };
}
