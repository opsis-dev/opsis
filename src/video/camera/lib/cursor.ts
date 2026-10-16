import {
  CLICK_SQUASH,
  CLICK_SQUASH_S,
  CURSOR_FADE_S,
  RIPPLE_S,
  type ClickEvent,
  type CursorSample,
} from "./click-log";
import { clamp, mix } from "./camera";

export type CursorState = {
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

  let lo = 0;
  let hi = track.length - 1;
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1;
    if (track[mid].t <= tMs) lo = mid;
    else hi = mid;
  }
  const a = track[lo];
  const b = track[hi];
  const span = b.t - a.t;
  const p = span < 1e-6 ? 0 : (tMs - a.t) / span;
  return { x: mix(a.x, b.x, p), y: mix(a.y, b.y, p) };
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
    if (c.tDownMs == null) continue;
    const since = tS - c.tDownMs / 1000;
    if (since < 0) continue;
    const held = c.tUpMs != null ? Math.max(0, (c.tUpMs - c.tDownMs) / 1000) : 0;
    const released = since - held;
    if (released <= 0) squash = Math.min(squash, CLICK_SQUASH);
    else if (released <= CLICK_SQUASH_S) {
      squash = Math.min(squash, mix(CLICK_SQUASH, 1, clamp(released / CLICK_SQUASH_S, 0, 1)));
    }
    if (since <= RIPPLE_S) ripple = clamp(since / RIPPLE_S, 0, 1);
  }

  return {
    x: here.x,
    y: here.y,
    squash,
    ripple,
    opacity: cursorOpacity(clicks, tS),
  };
}
