import { Easing } from "remotion";
import { DESIGN_WIDTH } from "./click-log";
import { clamp } from "../../../shared/math";

export const CAMERA_BEZIER = Easing.bezier(0.2, 0.2, 0.15, 1);

export const PUSH_BEZIER = Easing.bezier(0.15, 0.9, 0.75, 0.95);

export type CameraPose = {
  scale: number;
  cx: number;
  cy: number;
};

export const BASE_POSE: CameraPose = { scale: 1, cx: 0.5, cy: 0.48 };

export const mix = (a: number, b: number, t: number): number => a + (b - a) * t;

export function cameraEase(t01: number): number {
  return CAMERA_BEZIER(clamp(t01, 0, 1));
}

export function interpolatePose(a: CameraPose, b: CameraPose, t01: number): CameraPose {
  const p = cameraEase(t01);
  return {
    scale: mix(a.scale, b.scale, p),
    cx: mix(a.cx, b.cx, p),
    cy: mix(a.cy, b.cy, p),
  };
}

export function posesNearlyEqual(a: CameraPose, b: CameraPose): boolean {
  return (
    Math.abs(a.scale - b.scale) < 0.008 &&
    Math.abs(a.cx - b.cx) < 0.01 &&
    Math.abs(a.cy - b.cy) < 0.01
  );
}

export function moveDuration(
  from: CameraPose,
  to: CameraPose,
  kind: "in" | "out" | "pan",
  vp: { width: number; height: number },
  speed = 1,
): number {
  const dS = Math.abs(to.scale - from.scale);
  const dPx = poseTravelPx(from, to, vp);
  const ms = 480 + 420 * dS + 0.35 * dPx;
  const onScreen =
    kind === "out"
      ? clamp(ms * 1.25, 650, 1450)
      : kind === "pan"
        ? clamp(ms, 700, 1400)
        : clamp(ms, 550, 1250);
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

export const MAX_PAN_PX = 500;

export function contentToWindow(
  cx: number,
  cy: number,
  chromeFrac: number,
): { cx: number; cy: number } {
  if (chromeFrac <= 0) return { cx, cy };
  return { cx, cy: chromeFrac + cy * (1 - chromeFrac) };
}

export function poseToCss(
  pose: CameraPose,
  chromeFrac: number,
  fit = 1,
): { scale: number; translateX: number; translateY: number } {
  const S = pose.scale;
  const effective = fit * S;
  const mapped = contentToWindow(pose.cx, pose.cy, chromeFrac);
  const half = 1 / (2 * S);
  const Cx = clamp(mapped.cx, half, 1 - half);
  const Cy = clamp(mapped.cy, half, 1 - half);
  return {
    scale: effective,
    translateX: effective * (0.5 - Cx),
    translateY: effective * (0.5 - Cy),
  };
}
