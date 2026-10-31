import { equalizePaths, getPathBBox, pathToString, transformPath } from "svg-path-commander/util";
import { isNumber } from "#video/common/guards.ts";
import { clamp01 } from "#video/common/math.ts";

type EqualizedPath = ReturnType<typeof equalizePaths>[number];

export type MorphPair = { from: EqualizedPath; to: EqualizedPath; closed: boolean };

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
  const box = getPathBBox(d);
  const sourceWidth = Math.max(1e-6, box.width);
  const sourceHeight = Math.max(1e-6, box.height);
  const scaleX = width / sourceWidth;
  const scaleY = height / sourceHeight;
  const [sx, sy] = preserveAspectRatio
    ? [Math.min(scaleX, scaleY), Math.min(scaleX, scaleY)]
    : [scaleX, scaleY];
  const left = x + (width - sourceWidth * sx) / 2;
  const top = y + (height - sourceHeight * sy) / 2;
  return pathToString(
    transformPath(d, {
      translate: [left - box.x * sx, top - box.y * sy],
      scale: [sx, sy],
      origin: [0, 0],
    }),
    "off",
  );
}

function isClosed(d: string): boolean {
  return /z\s*$/i.test(d);
}

export function prepareMorph(from: string, to: string, fit?: FitToBoxOptions): MorphPair {
  const [fromPath, toPath] = equalizePaths(
    fit ? fitToBox(from, fit) : from,
    fit ? fitToBox(to, fit) : to,
  );
  return { from: fromPath, to: toPath, closed: isClosed(from) && isClosed(to) };
}

function numbersOf(segment: EqualizedPath[number]): number[] {
  return segment.filter(isNumber);
}

function blended(t: number, from: EqualizedPath, to: EqualizedPath): string {
  return from
    .map((segment, i) => {
      const other = numbersOf(to[i] ?? segment);
      const values = numbersOf(segment).map(
        (value, j) => value * (1 - t) + (other[j] ?? value) * t,
      );
      return `${segment[0]}${values.join(" ")}`;
    })
    .join(" ");
}

export function morphPath(progress: number, pair: MorphPair): string {
  const t = clamp01(progress);
  const drawn = blended(t, pair.from, pair.to);
  return pair.closed ? `${drawn} Z` : drawn;
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
