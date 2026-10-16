export type Point = { x: number; y: number };

export const GLIDE_PX_PER_MS = 0.55;
export const OVERSHOOT_THRESHOLD_PX = 500;
export const OVERSHOOT_PX = 26;
export const DRAG_MIN_MS = 600;
export const DRAG_MAX_MS = 2500;
export const MAX_PATH_POINTS = 64;

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

export type Glide = {
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
    travelMs != null && travelMs > 0 ? travelMs : clamp(dist / (GLIDE_PX_PER_MS * k), 450, 1350);
  const overshooting = dist > OVERSHOOT_THRESHOLD_PX * k;
  const aim = overshooting
    ? {
        x: clamp(
          to.x + ((to.x - from.x) / dist) * OVERSHOOT_PX * k,
          1,
          (viewport?.width ?? to.x) - 1,
        ),
        y: clamp(
          to.y + ((to.y - from.y) / dist) * OVERSHOOT_PX * k,
          1,
          (viewport?.height ?? to.y) - 1,
        ),
      }
    : to;
  return { durationMs, aim, overshooting, at: bulgedCurve(from, to, aim) };
}

export function bulgedCurve(from: Point, to: Point, aim: Point = to): (p: number) => Point {
  const dist = Math.hypot(to.x - from.x, to.y - from.y);
  if (dist < 1e-6) return () => ({ ...to });
  const nx = -(to.y - from.y) / dist;
  const ny = (to.x - from.x) / dist;
  const side = (to.x - from.x) * (to.y - from.y) >= 0 ? 1 : -1;
  const bulge = side * dist * 0.1;
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

export function pathLength(points: readonly Point[]): number {
  let total = 0;
  for (let i = 1; i < points.length; i++)
    total += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
  return total;
}

export function polylineAt(points: readonly Point[]): (p: number) => Point {
  if (points.length === 0) throw new Error("A path needs at least one point.");
  const total = pathLength(points);
  return (p) => {
    if (total < 1e-6 || p <= 0) return { ...points[0] };
    let left = clamp(p, 0, 1) * total;
    for (let i = 1; i < points.length; i++) {
      const a = points[i - 1];
      const b = points[i];
      const seg = Math.hypot(b.x - a.x, b.y - a.y);
      if (left <= seg && seg > 0) {
        const u = left / seg;
        return { x: a.x + (b.x - a.x) * u, y: a.y + (b.y - a.y) * u };
      }
      left -= seg;
    }
    return { ...points[points.length - 1] };
  };
}

export function dragDuration(points: readonly Point[], k: number, dragMs = 0): number {
  if (dragMs > 0) return dragMs;
  return clamp(pathLength(points) / (GLIDE_PX_PER_MS * k), DRAG_MIN_MS, DRAG_MAX_MS);
}

export function nudgeToward(from: Point, to: Point, px: number): Point {
  const dist = Math.hypot(to.x - from.x, to.y - from.y);
  if (dist < 1e-6) return { x: from.x + px, y: from.y };
  const step = Math.min(px, dist);
  return {
    x: from.x + ((to.x - from.x) / dist) * step,
    y: from.y + ((to.y - from.y) / dist) * step,
  };
}

export function parsePath(text: string): Point[] {
  const points: Point[] = [];
  text.split("\n").forEach((line, i) => {
    if (!line.trim()) return;
    const m = /^\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*$/.exec(line);
    if (!m) throw new Error(`Line ${i + 1} of the path isn't two numbers.`);
    points.push({ x: Number(m[1]), y: Number(m[2]) });
  });
  if (points.length === 0) throw new Error("Add at least one point to the path.");
  return points;
}

export function offsetPath(start: Point, offsets: readonly Point[]): Point[] {
  const points = offsets.map((o) => ({ x: start.x + o.x, y: start.y + o.y }));
  const first = points[0];
  return first && first.x === start.x && first.y === start.y ? points : [start, ...points];
}

export function thinPath(points: readonly Point[], max = MAX_PATH_POINTS): Point[] {
  const rounded = points.map((p) => ({ x: Math.round(p.x), y: Math.round(p.y) }));
  if (rounded.length <= max) return rounded;
  return Array.from(
    { length: max },
    (_, i) => rounded[Math.round((i * (rounded.length - 1)) / (max - 1))],
  );
}

export function boundsOf(points: readonly Point[]): { x: number; y: number; w: number; h: number } {
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  return { x, y, w: Math.max(...xs) - x, h: Math.max(...ys) - y };
}
