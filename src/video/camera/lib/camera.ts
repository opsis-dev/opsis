import { Easing } from "remotion";
import { clamp, clamp01, lerp } from "../../../shared/math";
import type { Viewport } from "../../../shared/schema/actions";
import { designScaleFor } from "./design-width";

const CAMERA_BEZIER = Easing.bezier(0.2, 0.2, 0.15, 1);

export type CameraPose = {
  scale: number;
  cx: number;
  cy: number;
};

export type ZoomState = {
  scale: number;
  translateX: number;
  translateY: number;
};

export const BASE_POSE: CameraPose = { scale: 1, cx: 0.5, cy: 0.48 };

export function cameraEase(t01: number): number {
  return CAMERA_BEZIER(clamp01(t01));
}

export function interpolatePose(a: CameraPose, b: CameraPose, t01: number): CameraPose {
  const p = cameraEase(t01);
  return {
    scale: lerp(a.scale, b.scale, p),
    cx: lerp(a.cx, b.cx, p),
    cy: lerp(a.cy, b.cy, p),
  };
}

const SAME_POSE = { scale: 0.008, center: 0.01 } as const;

export function posesNearlyEqual(a: CameraPose, b: CameraPose): boolean {
  return (
    Math.abs(a.scale - b.scale) < SAME_POSE.scale &&
    Math.abs(a.cx - b.cx) < SAME_POSE.center &&
    Math.abs(a.cy - b.cy) < SAME_POSE.center
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
  vp: Viewport,
  speed = 1,
): number {
  const dS = Math.abs(to.scale - from.scale);
  const dPx = poseTravelPx(from, to, vp);
  const ms = MOVE_BASE_MS + MOVE_MS_PER_SCALE * dS + MOVE_MS_PER_PX * dPx;
  const limit = MOVE_LIMITS[kind];
  const onScreen = clamp(ms * limit.stretch, limit.minMs, limit.maxMs);
  return (onScreen * speed) / 1000;
}

export function poseTravelPx(from: CameraPose, to: CameraPose, vp: Viewport): number {
  return (
    Math.hypot((to.cx - from.cx) * vp.width, (to.cy - from.cy) * vp.height) /
    designScaleFor(vp.width)
  );
}

export function poseToCss(pose: CameraPose, chromeFrac: number, fit = 1): ZoomState {
  const { scale } = pose;
  const effective = fit * scale;
  const windowCy = chromeFrac > 0 ? chromeFrac + pose.cy * (1 - chromeFrac) : pose.cy;
  const half = 1 / (2 * scale);
  const focusX = clamp(pose.cx, half, 1 - half);
  const focusY = clamp(windowCy, half, 1 - half);
  return {
    scale: effective,
    translateX: effective * (0.5 - focusX),
    translateY: effective * (0.5 - focusY),
  };
}
