import { isRecord } from "#video/common/guards.ts";
import { lerp } from "#video/common/math.ts";
import type { Point, Rect } from "#video/generated/spec.ts";

export type Bounds = { x: number; y: number; width: number; height: number };

export const boundsFromRect = (r: Rect): Bounds => ({ x: r.x, y: r.y, width: r.w, height: r.h });

const POINT_BOX = { w: 120, h: 48 };

export const pointRect = (point: Point, scale = 1): Rect => ({
  x: point.x - (POINT_BOX.w / 2) * scale,
  y: point.y - (POINT_BOX.h / 2) * scale,
  w: POINT_BOX.w * scale,
  h: POINT_BOX.h * scale,
});

export const rectCenter = (r: Rect): Point => ({
  x: r.x + r.w / 2,
  y: r.y + r.h / 2,
});

export const distance = (a: Point, b: Point): number => Math.hypot(b.x - a.x, b.y - a.y);

export const lerpPoint = (a: Point, b: Point, t: number): Point => ({
  x: lerp(a.x, b.x, t),
  y: lerp(a.y, b.y, t),
});

export function rectUnion(a: Rect, b: Rect): Rect {
  const x = Math.min(a.x, b.x);
  const y = Math.min(a.y, b.y);
  return {
    x,
    y,
    w: Math.max(a.x + a.w, b.x + b.w) - x,
    h: Math.max(a.y + a.h, b.y + b.h) - y,
  };
}

const isBounds = (value: unknown): value is Bounds =>
  isRecord(value) && ["x", "y", "width", "height"].every((key) => typeof value[key] === "number");

export const isBoundsList = (value: unknown): value is Bounds[] =>
  Array.isArray(value) && value.every(isBounds);
