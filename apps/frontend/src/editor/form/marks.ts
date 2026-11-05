import { isString } from "@opsis/video/common/guards";
import { clamp, numberOr, round3 } from "@opsis/video/common/math";
import type { ParamValues } from "./types";

export const MARK_KINDS = ["arrow", "rectangle", "oval", "step", "text"] as const;

export type MarkKind = (typeof MARK_KINDS)[number];

export type MarkTool = "select" | MarkKind;

export type Point = { x: number; y: number };

export type MarkPoints = { x: number; y: number; x2: number; y2: number };

const MIN_DRAG = 0.01;

function isMarkKind(v: unknown): v is MarkKind {
  return isString(v) && (MARK_KINDS as readonly string[]).includes(v);
}

export function kindOf(mark: ParamValues): MarkKind {
  return isMarkKind(mark.kind) ? mark.kind : "arrow";
}

export function twoEnded(kind: MarkKind): boolean {
  return kind === "arrow" || kind === "rectangle" || kind === "oval";
}

function unit(v: number): number {
  return round3(clamp(v, 0, 1));
}

export function pointsOf(mark: ParamValues): MarkPoints {
  return {
    x: numberOr(mark.x, 0),
    y: numberOr(mark.y, 0),
    x2: numberOr(mark.x2, 0),
    y2: numberOr(mark.y2, 0),
  };
}

export function moved(points: MarkPoints, dx: number, dy: number): MarkPoints {
  const left = Math.min(points.x, points.x2);
  const right = Math.max(points.x, points.x2);
  const top = Math.min(points.y, points.y2);
  const bottom = Math.max(points.y, points.y2);
  const sx = clamp(dx, -left, 1 - right);
  const sy = clamp(dy, -top, 1 - bottom);
  return {
    x: unit(points.x + sx),
    y: unit(points.y + sy),
    x2: unit(points.x2 + sx),
    y2: unit(points.y2 + sy),
  };
}

export function withEnd(points: MarkPoints, end: 0 | 1, at: Point): MarkPoints {
  return end === 0
    ? { ...points, x: unit(at.x), y: unit(at.y) }
    : { ...points, x2: unit(at.x), y2: unit(at.y) };
}

export function drawn(kind: MarkKind, start: Point, end: Point): MarkPoints | null {
  if (!twoEnded(kind)) return { x: unit(end.x), y: unit(end.y), x2: unit(end.x), y2: unit(end.y) };
  if (Math.hypot(end.x - start.x, end.y - start.y) < MIN_DRAG) return null;
  return { x: unit(start.x), y: unit(start.y), x2: unit(end.x), y2: unit(end.y) };
}

export function stepNumbers(
  marks: readonly ParamValues[],
  first: number,
): Array<number | undefined> {
  let next = first;
  return marks.map((m) => (kindOf(m) === "step" ? next++ : undefined));
}
