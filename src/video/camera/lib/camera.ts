import { Easing } from "remotion";
import { DESIGN_WIDTH } from "./click-log";
import { clamp, lerp } from "../../../shared/math";

const CAMERA_BEZIER = Easing.bezier(0.2, 0.2, 0.15, 1);

export type CameraPose = {
  scale: number;
  cx: number;
  cy: number;
};

export const BASE_POSE: CameraPose = { scale: 1, cx: 0.5, cy: 0.48 };

export function partitionPoint<T>(items: readonly T[], before: (item: T) => boolean): number {
  let lo = 0;
  let hi = items.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (before(items[mid])) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

export function cameraEase(t01: number): number {
  return CAMERA_BEZIER(clamp(t01, 0, 1));
}

export function interpolatePose(a: CameraPose, b: CameraPose, t01: number): CameraPose {
  const p = cameraEase(t01);
  return {
    scale: lerp(a.scale, b.scale, p),
    cx: lerp(a.cx, b.cx, p),
    cy: lerp(a.cy, b.cy, p),
  };
}

export function posesNearlyEqual(a: CameraPose, b: CameraPose): boolean {
  return (
    Math.abs(a.scale - b.scale) < 0.008 &&
    Math.abs(a.cx - b.cx) < 0.01 &&
    Math.abs(a.cy - b.cy) < 0.01
  );
}

const MOVE_BASE_MS = 480;
const MOVE_MS_PER_SCALE = 420;
const MOVE_MS_PER_PX = 0.35;

const MOVE_LIMITS = {
  in: { stretch: 1, minMs: 550, maxMs: 1250 },
  out: { stretch: 1.25, minMs: 650, maxMs: 1450 },
  pan: { stretch: 1, minMs: 700, maxMs: 1400 },
} as const;

export function moveDuration(
  from: CameraPose,
  to: CameraPose,
  kind: keyof typeof MOVE_LIMITS,
  vp: { width: number; height: number },
  speed = 1,
): number {
  const dS = Math.abs(to.scale - from.scale);
  const dPx = poseTravelPx(from, to, vp);
  const ms = MOVE_BASE_MS + MOVE_MS_PER_SCALE * dS + MOVE_MS_PER_PX * dPx;
  const limit = MOVE_LIMITS[kind];
  const onScreen = clamp(ms * limit.stretch, limit.minMs, limit.maxMs);
  return (onScreen * speed) / 1000;
}

export function poseTravelPx(
  from: CameraPose,
  to: CameraPose,
  vp: { width: number; height: number },
): number {
  const k = vp.width > 0 ? vp.width / DESIGN_WIDTH : 1;
  return Math.hypot((to.cx - from.cx) * vp.width, (to.cy - from.cy) * vp.height) / k;
}

export function poseToCss(
  pose: CameraPose,
  chromeFrac: number,
  fit = 1,
): { scale: number; translateX: number; translateY: number } {
  const S = pose.scale;
  const effective = fit * S;
  const windowCy = chromeFrac > 0 ? chromeFrac + pose.cy * (1 - chromeFrac) : pose.cy;
  const half = 1 / (2 * S);
  const Cx = clamp(pose.cx, half, 1 - half);
  const Cy = clamp(windowCy, half, 1 - half);
  return {
    scale: effective,
    translateX: effective * (0.5 - Cx),
    translateY: effective * (0.5 - Cy),
  };
}
