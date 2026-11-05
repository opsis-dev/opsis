import { getPathBBox, getPointAtLength, getTotalLength } from "svg-path-commander/util";

export type PathPoint = {
  x: number;
  y: number;
};

export type PathSample = PathPoint & {
  angle: number;
};

export type PathBox = { x1: number; y1: number; x2: number; y2: number };

const lengthCache = new Map<string, number>();
const boxCache = new Map<string, PathBox>();
const TANGENT_STEP = 1e-4;

export function pathLength(d: string): number {
  const cached = lengthCache.get(d);
  if (cached !== undefined) {
    return cached;
  }
  const length = getTotalLength(d);
  lengthCache.set(d, length);
  return length;
}

export function pathBoundingBox(d: string): PathBox {
  const cached = boxCache.get(d);
  if (cached !== undefined) {
    return cached;
  }
  const { x, y, x2, y2 } = getPathBBox(d);
  const box = { x1: x, y1: y, x2, y2 };
  boxCache.set(d, box);
  return box;
}

export function clampProgress(progress: number) {
  return Math.min(1, Math.max(0, progress));
}

export function getPathDrawStyles(progress: number, path: string) {
  const shown = clampProgress(progress);
  const length = pathLength(path);
  const dash = shown === 0 ? length * 1.5 : length;
  return {
    strokeDasharray: `${dash} ${dash}`,
    strokeDashoffset: shown === 0 ? dash : length - shown * length,
  };
}

export function fitViewBox(paths: string | string[], padding = 0): string {
  const list = (Array.isArray(paths) ? paths : [paths]).filter(Boolean);
  const boxes = list.map(pathBoundingBox);

  if (boxes.length === 0) {
    return "0 0 100 100";
  }

  const x1 = Math.min(...boxes.map((box) => box.x1));
  const y1 = Math.min(...boxes.map((box) => box.y1));
  const x2 = Math.max(...boxes.map((box) => box.x2));
  const y2 = Math.max(...boxes.map((box) => box.y2));

  return [
    x1 - padding,
    y1 - padding,
    Math.max(1, x2 - x1 + padding * 2),
    Math.max(1, y2 - y1 + padding * 2),
  ].join(" ");
}

function directionAt(d: string, at: number, length: number): PathPoint {
  const step = Math.max(length * TANGENT_STEP, 1e-6);
  const from = Math.min(at, length - step);
  const here = getPointAtLength(d, from);
  const ahead = getPointAtLength(d, from + step);
  return { x: ahead.x - here.x, y: ahead.y - here.y };
}

export function samplePath(d: string, progress: number): PathSample {
  const length = pathLength(d);
  const at = clampProgress(progress) * length;
  const point = getPointAtLength(d, at);
  const direction = length > 0 ? directionAt(d, at, length) : { x: 1, y: 0 };

  return {
    x: point.x,
    y: point.y,
    angle: (Math.atan2(direction.y, direction.x) * 180) / Math.PI,
  };
}

export function waypointsToPath(points: readonly PathPoint[], smoothing = 0.25): string {
  if (points.length === 0) {
    return "";
  }
  if (points.length === 1) {
    return `M ${points[0].x} ${points[0].y}`;
  }

  const segments = [`M ${points[0].x} ${points[0].y}`];

  for (let index = 0; index < points.length - 1; index += 1) {
    const previous = points[index - 1] ?? points[index];
    const from = points[index];
    const to = points[index + 1];
    const next = points[index + 2] ?? to;

    if (smoothing <= 0) {
      segments.push(`L ${to.x} ${to.y}`);
      continue;
    }

    const c1 = {
      x: from.x + ((to.x - previous.x) / 2) * smoothing,
      y: from.y + ((to.y - previous.y) / 2) * smoothing,
    };
    const c2 = {
      x: to.x - ((next.x - from.x) / 2) * smoothing,
      y: to.y - ((next.y - from.y) / 2) * smoothing,
    };

    segments.push(`C ${c1.x} ${c1.y} ${c2.x} ${c2.y} ${to.x} ${to.y}`);
  }

  return segments.join(" ");
}
