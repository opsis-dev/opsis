import { isRecord } from "#video/common/guards.ts";
import { lerp } from "#video/common/math.ts";
import type { Bounds, Point, Rect } from "#video/generated/spec.ts";

export function boundsFromRect(r: Rect): Bounds {
  return { x: r.x, y: r.y, width: r.w, height: r.h };
}

const POINT_BOX = { w: 120, h: 48 };

export function pointRect(point: Point, scale = 1): Rect {
  return {
    x: point.x - (POINT_BOX.w / 2) * scale,
    y: point.y - (POINT_BOX.h / 2) * scale,
    w: POINT_BOX.w * scale,
    h: POINT_BOX.h * scale,
  };
}

export function rectCenter(r: Rect): Point {
  return {
    x: r.x + r.w / 2,
    y: r.y + r.h / 2,
  };
}

export function distance(a: Point, b: Point): number {
  return Math.hypot(b.x - a.x, b.y - a.y);
}

export function lerpPoint(a: Point, b: Point, t: number): Point {
  return {
    x: lerp(a.x, b.x, t),
    y: lerp(a.y, b.y, t),
  };
}

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

function isBounds(value: unknown): value is Bounds {
  return (
    isRecord(value) && ["x", "y", "width", "height"].every((key) => typeof value[key] === "number")
  );
}

export function isBoundsList(value: unknown): value is Bounds[] {
  return Array.isArray(value) && value.every(isBounds);
}

export function polar(cx: number, cy: number, radius: number, degrees: number): Point {
  const radians = ((degrees - 90) * Math.PI) / 180;
  return { x: cx + radius * Math.cos(radians), y: cy + radius * Math.sin(radians) };
}

export function arcPath(
  cx: number,
  cy: number,
  radius: number,
  startAngle: number,
  endAngle: number,
): string {
  if (endAngle - startAngle <= 0.001) return "";
  const start = polar(cx, cy, radius, startAngle);
  const middle = polar(cx, cy, radius, (startAngle + endAngle) / 2);
  const end = polar(cx, cy, radius, endAngle);
  return `M ${start.x} ${start.y} A ${radius} ${radius} 0 0 1 ${middle.x} ${middle.y} A ${radius} ${radius} 0 0 1 ${end.x} ${end.y}`;
}
