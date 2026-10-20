import { isNonEmpty } from "../../shared/lists";
import { clamp, EPSILON, sum } from "../../shared/math";
import {
  distance,
  lerpPoint,
  readPath,
  type Point,
  type Rect,
  type Viewport,
} from "../../shared/schema/actions";

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
  designScale: number,
  viewport: Viewport | null,
  travelMs?: number,
): Glide {
  const dist = distance(from, to);
  const durationMs =
    travelMs != null && travelMs > 0
      ? travelMs
      : clamp(dist / (GLIDE_PX_PER_MS * designScale), GLIDE_MIN_MS, GLIDE_MAX_MS);
  const overshooting = dist > OVERSHOOT_THRESHOLD_PX * designScale;
  const past = (a: number, b: number, size: number | undefined) =>
    clamp(b + ((b - a) / dist) * OVERSHOOT_PX * designScale, 1, (size ?? b) - 1);
  const aim = overshooting
    ? { x: past(from.x, to.x, viewport?.width), y: past(from.y, to.y, viewport?.height) }
    : to;
  return { durationMs, aim, overshooting, at: bulgedCurve(from, to, aim) };
}

export function bulgedCurve(from: Point, to: Point, aim: Point = to): (p: number) => Point {
  const dist = distance(from, to);
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
  points.slice(1).map((b, i) => distance(points[i], b));

export const lastPoint = (points: readonly [Point, ...Point[]]): Point => points[points.length - 1];

export function polylineAt(points: readonly Point[]): (p: number) => Point {
  if (!isNonEmpty(points)) throw new Error("A path needs at least one point.");
  const [first] = points;
  const last = lastPoint(points);
  const lengths = segmentLengths(points);
  const total = sum(lengths);
  return (p) => {
    if (total < EPSILON || p <= 0) return { ...first };
    let left = Math.min(p, 1) * total;
    for (const [i, seg] of lengths.entries()) {
      if (left <= seg && seg > 0) {
        return lerpPoint(points[i], points[i + 1], left / seg);
      }
      left -= seg;
    }
    return { ...last };
  };
}

export function dragDuration(points: readonly Point[], designScale: number, dragMs = 0): number {
  if (dragMs > 0) return dragMs;
  return clamp(
    sum(segmentLengths(points)) / (GLIDE_PX_PER_MS * designScale),
    DRAG_MIN_MS,
    DRAG_MAX_MS,
  );
}

export function nudgeToward(from: Point, to: Point, px: number): Point {
  const dist = distance(from, to);
  if (dist < EPSILON) return { x: from.x + px, y: from.y };
  const step = Math.min(px, dist);
  return {
    x: from.x + ((to.x - from.x) / dist) * step,
    y: from.y + ((to.y - from.y) / dist) * step,
  };
}

export function parsePath(text: string): [Point, ...Point[]] {
  const path = readPath(text);
  if (!path.ok) throw new Error(`${path.problem}.`);
  return path.points;
}

export function offsetPath(start: Point, offsets: readonly Point[]): [Point, ...Point[]] {
  const points = offsets.map((o) => ({ x: start.x + o.x, y: start.y + o.y }));
  const [first, ...rest] = points;
  return first && first.x === start.x && first.y === start.y
    ? [first, ...rest]
    : [start, ...points];
}

export function thinPath(points: readonly Point[]): Point[] {
  const rounded = points.map((p) => ({ x: Math.round(p.x), y: Math.round(p.y) }));
  if (rounded.length <= MAX_PATH_POINTS) return rounded;
  return Array.from(
    { length: MAX_PATH_POINTS },
    (_, i) => rounded[Math.round((i * (rounded.length - 1)) / (MAX_PATH_POINTS - 1))],
  );
}

export function boundsOf(points: readonly Point[]): Rect {
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  return { x, y, w: Math.max(...xs) - x, h: Math.max(...ys) - y };
}
