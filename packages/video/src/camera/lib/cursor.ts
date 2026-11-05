import type { ClickEvent, CursorSample } from "#video/camera/lib/click-log.ts";
import { lerpPoint } from "#video/common/geometry.ts";
import { clamp01, lerp, partitionPoint, sampleKeyed } from "#video/common/math.ts";
import type { Point } from "#video/generated/spec.ts";

const CLICK_SQUASH_S = 0.12;
const CURSOR_FADE_S = 0.18;
const ARROW_SHAPE = "default";

export type CursorMotion = { pressSquash: number; hideWhileTyping: boolean; rippleS: number };

const DEFAULT_CURSOR_MOTION: CursorMotion = {
  pressSquash: 0.9,
  hideWhileTyping: true,
  rippleS: 0.35,
};

export type Ring = { progress: number; hollow: boolean };

type CursorState = {
  x: number;
  y: number;
  squash: number;
  rings: Ring[];
  opacity: number;
  shape: string;
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

function trackPointAt(track: CursorSample[], tS: number): Point | undefined {
  return sampleKeyed(
    track,
    tS * 1000,
    (s) => s.t,
    (s) => ({ x: s.x, y: s.y }),
    lerpPoint,
  );
}

function shapeAt(track: CursorSample[], tS: number): string {
  for (let i = partitionPoint(track, (s) => s.t <= tS * 1000) - 1; i >= 0; i--) {
    const shape = track[i].c;
    if (shape !== undefined) return shape;
  }
  return ARROW_SHAPE;
}

function squashAfter(releasedS: number, pressed: number): number {
  if (releasedS <= 0) return pressed;
  if (releasedS > CLICK_SQUASH_S) return 1;
  return lerp(pressed, 1, releasedS / CLICK_SQUASH_S);
}

function rippleFor(sinceS: number, rippleS: number): number | null {
  return sinceS <= rippleS && rippleS > 0 ? clamp01(sinceS / rippleS) : null;
}

export function cursorAt(
  track: CursorSample[],
  clicks: ClickEvent[],
  tS: number,
  motion: CursorMotion = DEFAULT_CURSOR_MOTION,
): CursorState | null {
  const here = trackPointAt(track, tS);
  if (!here) return null;

  let squash = 1;
  const rings: Ring[] = [];
  for (const c of clicks) {
    const presses = [c.tDownMs, c.tSecondDownMs].filter((t) => t != null);
    for (const pressMs of presses) {
      const sinceS = tS - pressMs / 1000;
      if (sinceS < 0) continue;
      const heldS = c.tUpMs != null ? Math.max(0, (c.tUpMs - pressMs) / 1000) : 0;
      squash = Math.min(squash, squashAfter(sinceS - heldS, motion.pressSquash));
      const progress = rippleFor(sinceS, motion.rippleS);
      if (progress !== null) rings.push({ progress, hollow: c.button === "right" });
    }
  }

  return {
    x: here.x,
    y: here.y,
    squash,
    rings,
    opacity: motion.hideWhileTyping ? cursorOpacity(clicks, tS) : 1,
    shape: shapeAt(track, tS),
  };
}
