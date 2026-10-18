import type { ClickEvent, CursorSample } from "./click-log";
import { clamp01, lerp, sampleKeyed } from "../../../shared/math";
import type { Point } from "../../../shared/schema/actions";

export const CLICK_SQUASH = 0.9;
export const CLICK_SQUASH_S = 0.12;
export const CURSOR_FADE_S = 0.18;
const RIPPLE_S = 0.35;

type CursorState = {
  x: number;
  y: number;
  squash: number;
  ripple: number | null;
  opacity: number;
};

function cursorOpacity(clicks: ClickEvent[], tS: number): number {
  let opacity = 1;
  for (const c of clicks) {
    if (c.typeEndMs == null) continue;
    const from = c.tMs / 1000;
    const to = c.typeEndMs / 1000;
    if (tS <= from - CURSOR_FADE_S || tS >= to + CURSOR_FADE_S) continue;
    const fadingOut = clamp01((from - tS) / CURSOR_FADE_S);
    const fadingIn = clamp01((tS - to) / CURSOR_FADE_S);
    opacity = Math.min(opacity, Math.max(fadingOut, fadingIn));
  }
  return opacity;
}

const trackPointAt = (track: CursorSample[], tS: number): Point | undefined =>
  sampleKeyed(
    track,
    tS * 1000,
    (s) => s.t,
    (s) => ({ x: s.x, y: s.y }),
    (a, b, p) => ({ x: lerp(a.x, b.x, p), y: lerp(a.y, b.y, p) }),
  );

function squashAfter(releasedS: number): number {
  if (releasedS <= 0) return CLICK_SQUASH;
  if (releasedS > CLICK_SQUASH_S) return 1;
  return lerp(CLICK_SQUASH, 1, clamp01(releasedS / CLICK_SQUASH_S));
}

const rippleFor = (sinceS: number): number | null =>
  sinceS <= RIPPLE_S ? clamp01(sinceS / RIPPLE_S) : null;

export function cursorAt(
  track: CursorSample[],
  clicks: ClickEvent[],
  tS: number,
): CursorState | null {
  const here = trackPointAt(track, tS);
  if (!here) return null;

  let squash = 1;
  let ripple: number | null = null;
  for (const c of clicks) {
    if (c.tDownMs == null) continue;
    const sinceS = tS - c.tDownMs / 1000;
    if (sinceS < 0) continue;
    const heldS = c.tUpMs != null ? Math.max(0, (c.tUpMs - c.tDownMs) / 1000) : 0;
    squash = Math.min(squash, squashAfter(sinceS - heldS));
    ripple = rippleFor(sinceS) ?? ripple;
  }

  return {
    x: here.x,
    y: here.y,
    squash,
    ripple,
    opacity: cursorOpacity(clicks, tS),
  };
}
