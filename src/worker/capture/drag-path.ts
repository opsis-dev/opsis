import { clamp, EPSILON, sum } from "../../shared/math";
import type { Point } from "../../shared/schema/actions";

const GLIDE_PX_PER_MS = 0.55;
const GLIDE_MIN_MS = 450;
const GLIDE_MAX_MS = 1350;
const OVERSHOOT_THRESHOLD_PX = 500;
const OVERSHOOT_PX = 26;
const BULGE = 0.1;
const DRAG_MIN_MS = 600;
const DRAG_MAX_MS = 2500;
const MAX_PATH_POINTS = 64;

type Glide = {
  durationMs: number;
  aim: Point;
  overshooting: boolean;
  at: (p: number) => Point;
};

export function glidePlan(
  from: Point,
  to: Point,
  k: number,
  viewport: { width: number; height: number } | null,
  travelMs?: number,
): Glide {
  const dist = Math.hypot(to.x - from.x, to.y - from.y);
  const durationMs =
    travelMs != null && travelMs > 0
      ? travelMs
      : clamp(dist / (GLIDE_PX_PER_MS * k), GLIDE_MIN_MS, GLIDE_MAX_MS);
  const overshooting = dist > OVERSHOOT_THRESHOLD_PX * k;
  const past = (a: number, b: number, size: number | undefined) =>
    clamp(b + ((b - a) / dist) * OVERSHOOT_PX * k, 1, (size ?? b) - 1);
  const aim = overshooting
    ? { x: past(from.x, to.x, viewport?.width), y: past(from.y, to.y, viewport?.height) }
    : to;
  return { durationMs, aim, overshooting, at: bulgedCurve(from, to, aim) };
}

export function bulgedCurve(from: Point, to: Point, aim: Point = to): (p: number) => Point {
  const dist = Math.hypot(to.x - from.x, to.y - from.y);
  if (dist < EPSILON) return () => ({ ...to });
  const nx = -(to.y - from.y) / dist;
  const ny = (to.x - from.x) / dist;
  const side = (to.x - from.x) * (to.y - from.y) >= 0 ? 1 : -1;
  const bulge = side * dist * BULGE;
  const cpx = (from.x + aim.x) / 2 + nx * bulge;
  const cpy = (from.y + aim.y) / 2 + ny * bulge;
  return (p) => {
    const u = 1 - p;
    return {
      x: u * u * from.x + 2 * u * p * cpx + p * p * aim.x,
      y: u * u * from.y + 2 * u * p * cpy + p * p * aim.y,
    };
  };
}

const segmentLengths = (points: readonly Point[]): number[] =>
  points.slice(1).map((b, i) => Math.hypot(b.x - points[i].x, b.y - points[i].y));

export const lastPoint = (points: readonly Point[]): Point => points[points.length - 1];

export function polylineAt(points: readonly Point[]): (p: number) => Point {
  const first = points[0];
  if (!first) throw new Error("A path needs at least one point.");
  const last = lastPoint(points);
  const lengths = segmentLengths(points);
  const total = sum(lengths);
  return (p) => {
    if (total < EPSILON || p <= 0) return { ...first };
    let left = Math.min(p, 1) * total;
    for (const [i, seg] of lengths.entries()) {
      if (left <= seg && seg > 0) {
        const a = points[i];
        const b = points[i + 1];
        const u = left / seg;
        return { x: a.x + (b.x - a.x) * u, y: a.y + (b.y - a.y) * u };
      }
      left -= seg;
    }
    return { ...last };
  };
}

export function dragDuration(points: readonly Point[], k: number, dragMs = 0): number {
  if (dragMs > 0) return dragMs;
  return clamp(sum(segmentLengths(points)) / (GLIDE_PX_PER_MS * k), DRAG_MIN_MS, DRAG_MAX_MS);
}

export function nudgeToward(from: Point, to: Point, px: number): Point {
  const dist = Math.hypot(to.x - from.x, to.y - from.y);
  if (dist < EPSILON) return { x: from.x + px, y: from.y };
  const step = Math.min(px, dist);
  return {
    x: from.x + ((to.x - from.x) / dist) * step,
    y: from.y + ((to.y - from.y) / dist) * step,
  };
}

const POINT_TEXT = /^\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*$/;

export function parsePoint(text: string): Point | null {
  const m = POINT_TEXT.exec(text);
  return m ? { x: Number(m[1]), y: Number(m[2]) } : null;
}

export function parsePath(text: string): Point[] {
  const points: Point[] = [];
  text.split("\n").forEach((line, i) => {
    if (!line.trim()) return;
    const point = parsePoint(line);
    if (!point) throw new Error(`Line ${i + 1} of the path isn't two numbers.`);
    points.push(point);
  });
  if (points.length === 0) throw new Error("Add at least one point to the path.");
  return points;
}

export function offsetPath(start: Point, offsets: readonly Point[]): Point[] {
  const points = offsets.map((o) => ({ x: start.x + o.x, y: start.y + o.y }));
  const first = points[0];
  return first && first.x === start.x && first.y === start.y ? points : [start, ...points];
}

export function thinPath(points: readonly Point[]): Point[] {
  const rounded = points.map((p) => ({ x: Math.round(p.x), y: Math.round(p.y) }));
  if (rounded.length <= MAX_PATH_POINTS) return rounded;
  return Array.from(
    { length: MAX_PATH_POINTS },
    (_, i) => rounded[Math.round((i * (rounded.length - 1)) / (MAX_PATH_POINTS - 1))],
  );
}

export function boundsOf(points: readonly Point[]): { x: number; y: number; w: number; h: number } {
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  return { x, y, w: Math.max(...xs) - x, h: Math.max(...ys) - y };
}
