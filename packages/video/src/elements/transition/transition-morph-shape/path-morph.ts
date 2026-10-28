import {
  getBoundingBox,
  interpolatePath,
  normalizePath,
  reversePath,
  scalePath,
  translatePath,
} from "@remotion/paths";

export type MorphPair = { from: string; to: string };

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

const MAX_CACHE_ENTRIES = 256;
const alignmentCache = new Map<string, string>();

function cacheGet<T>(cache: Map<string, T>, key: string, build: () => T): T {
  const hit = cache.get(key);
  if (hit !== undefined) {
    return hit;
  }
  const value = build();
  if (cache.size >= MAX_CACHE_ENTRIES) {
    cache.clear();
  }
  cache.set(key, value);
  return value;
}

function signedArea(d: string): number {
  const points = [...d.matchAll(/(-?\d*\.?\d+)[,\s]+(-?\d*\.?\d+)/g)].map(
    (match) => [Number(match[1]), Number(match[2])] as const,
  );
  if (points.length < 3) {
    return 0;
  }
  let area = 0;
  for (let index = 0; index < points.length; index += 1) {
    const [x1, y1] = points[index];
    const [x2, y2] = points[(index + 1) % points.length];
    area += x1 * y2 - x2 * y1;
  }
  return area / 2;
}

export function alignForMorph(from: string, to: string): string {
  return cacheGet(alignmentCache, `${from}||${to}`, () => {
    const fromArea = signedArea(from);
    const toArea = signedArea(to);
    if (fromArea === 0 || toArea === 0) {
      return to;
    }
    return Math.sign(fromArea) === Math.sign(toArea) ? to : reversePath(to);
  });
}

export type FitToBoxOptions = {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  preserveAspectRatio?: boolean;
};

export function fitToBox(
  d: string,
  { x = 0, y = 0, width = 100, height = 100, preserveAspectRatio = true }: FitToBoxOptions = {},
): string {
  const box = getBoundingBox(d);
  const sourceWidth = Math.max(1e-6, box.x2 - box.x1);
  const sourceHeight = Math.max(1e-6, box.y2 - box.y1);

  const scaleX = width / sourceWidth;
  const scaleY = height / sourceHeight;
  const [sx, sy] = preserveAspectRatio
    ? [Math.min(scaleX, scaleY), Math.min(scaleX, scaleY)]
    : [scaleX, scaleY];

  const atOrigin = translatePath(d, -box.x1, -box.y1);
  const scaled = scalePath(atOrigin, sx, sy);

  const drawnWidth = sourceWidth * sx;
  const drawnHeight = sourceHeight * sy;

  return translatePath(scaled, x + (width - drawnWidth) / 2, y + (height - drawnHeight) / 2);
}

export type MorphOptions = {
  align?: boolean;
  fit?: FitToBoxOptions | false;
};

export function prepareMorph(
  from: string,
  to: string,
  { align = true, fit = false }: MorphOptions = {},
): MorphPair {
  const fittedFrom = fit ? fitToBox(normalizePath(from), fit) : normalizePath(from);
  const fittedTo = fit ? fitToBox(normalizePath(to), fit) : normalizePath(to);

  return {
    from: fittedFrom,
    to: align ? alignForMorph(fittedFrom, fittedTo) : fittedTo,
  };
}

export function morphPath(progress: number, pair: MorphPair): string {
  const t = clamp01(progress);
  if (t <= 0) {
    return pair.from;
  }
  if (t >= 1) {
    return pair.to;
  }
  return interpolatePath(t, pair.from, pair.to);
}

export function morphBetween(
  progress: number,
  from: string,
  to: string,
  options: MorphOptions = {},
): string {
  return morphPath(progress, prepareMorph(from, to, options));
}

export function prepareMorphSequence(
  paths: readonly string[],
  { loop = false, ...options }: MorphOptions & { loop?: boolean } = {},
): MorphPair[] {
  const list = paths.filter(Boolean);
  if (list.length < 2) {
    return [];
  }
  const chain = loop ? [...list, list[0]] : list;
  return chain.slice(0, -1).map((from, index) => prepareMorph(from, chain[index + 1], options));
}

export function morphSequence(progress: number, pairs: MorphPair[]): string {
  if (pairs.length === 0) {
    return "";
  }
  const t = clamp01(progress) * pairs.length;
  const segment = Math.min(pairs.length - 1, Math.floor(t));
  return morphPath(t - segment, pairs[segment]);
}

export const MORPH_SHAPES = {
  circle:
    "M 50 2 C 76.5 2 98 23.5 98 50 C 98 76.5 76.5 98 50 98 C 23.5 98 2 76.5 2 50 C 2 23.5 23.5 2 50 2 Z",
  square: "M 2 2 C 34 2 66 2 98 2 C 98 34 98 66 98 98 C 66 98 34 98 2 98 C 2 66 2 34 2 2 Z",
  squircle: "M 50 2 C 84 2 98 16 98 50 C 98 84 84 98 50 98 C 16 98 2 84 2 50 C 2 16 16 2 50 2 Z",
  triangle: "M 50 4 C 66 32 82 60 96 92 C 65 92 35 92 4 92 C 18 60 34 32 50 4 Z",
  diamond: "M 50 2 C 66 18 82 34 98 50 C 82 66 66 82 50 98 C 34 82 18 66 2 50 C 18 34 34 18 50 2 Z",
  blob: "M 50 6 C 76 10 94 26 92 52 C 90 78 70 96 48 94 C 24 92 6 74 8 48 C 10 24 26 4 50 6 Z",
} as const;

export type MorphShapeName = keyof typeof MORPH_SHAPES;
