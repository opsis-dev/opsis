import rough from "roughjs";
import { clamp } from "#video/common/math.ts";
import type { HandDrawnLook } from "#video/common/recording.ts";

export type MarkLook = HandDrawnLook | "crossed-off";

export type MarkBox = { x: number; y: number; w: number; h: number };

export type MarkShape = { look: MarkLook; seed: number; strokes: number; roughness?: number };

export type MarkPaths = { paths: string[]; strokeWidth?: number };

const ROUGH = rough.generator();

type Drawable = ReturnType<typeof ROUGH.line>;
type Op = Drawable["sets"][number]["ops"][number];
type Options = NonNullable<Parameters<typeof ROUGH.line>[4]>;
type Point = [number, number];

function strokeOptions(seed: number, marker: boolean, roughness?: number): Options {
  return {
    maxRandomnessOffset: 5,
    roughness: roughness ?? (marker ? 3 : 1.5),
    bowing: 1,
    curveTightness: 0,
    curveFitting: 0.95,
    curveStepCount: 9,
    disableMultiStroke: true,
    preserveVertices: false,
    seed,
  };
}

function backAndForth(
  strokes: number,
  [from, to]: [Point, Point],
  options: (stroke: number) => Options,
): Drawable[] {
  return Array.from({ length: strokes }, (_, i) => {
    const [start, end] = i % 2 ? [to, from] : [from, to];
    return ROUGH.line(start[0], start[1], end[0], end[1], options(i));
  });
}

function drawables({ look, seed, strokes, roughness }: MarkShape, box: MarkBox): Drawable[] {
  const { x, y, w, h } = box;
  function stroke(i: number) {
    return strokeOptions(seed + i, false, roughness);
  }
  switch (look) {
    case "box":
      return Array.from({ length: strokes }, (_, i) => ROUGH.rectangle(x, y, w, h, stroke(i)));
    case "circle":
      return Array.from({ length: strokes }, (_, i) =>
        ROUGH.ellipse(x + w / 2, y + h / 2, (w / Math.SQRT2) * 2, (h / Math.SQRT2) * 2, stroke(i)),
      );
    case "underline":
      return backAndForth(
        strokes,
        [
          [x, y + h],
          [x + w, y + h],
        ],
        stroke,
      );
    case "crossed-off":
      return [
        ...backAndForth(
          strokes,
          [
            [x, y],
            [x + w, y + h],
          ],
          stroke,
        ),
        ...backAndForth(
          strokes,
          [
            [x + w, y],
            [x, y + h],
          ],
          (i) => stroke(strokes + i),
        ),
      ];
    case "marker":
      return backAndForth(
        strokes,
        [
          [x, y + h / 2],
          [x + w, y + h / 2],
        ],
        (i) => strokeOptions(seed + i, true, roughness),
      );
  }
}

function opText({ op, data }: Op): string {
  switch (op) {
    case "move":
      return `M${data[0]} ${data[1]}`;
    case "lineTo":
      return `L${data[0]} ${data[1]}`;
    case "bcurveTo":
      return `C${data[0]} ${data[1]}, ${data[2]} ${data[3]}, ${data[4]} ${data[5]}`;
  }
}

function subpaths(ops: Op[]): string[] {
  return ops
    .reduce<Op[][]>((runs, op) => {
      if (op.op === "move" || runs.length === 0) runs.push([op]);
      else runs.at(-1)?.push(op);
      return runs;
    }, [])
    .map((run) => run.map(opText).join(" "));
}

export function markPaths(shape: MarkShape, box: MarkBox): MarkPaths {
  if (box.w === 0 || box.h === 0) return { paths: [] };
  const paths = drawables(shape, box)
    .flatMap((drawn) => drawn.sets)
    .flatMap((set) => subpaths(set.ops));
  return shape.look === "marker" ? { paths, strokeWidth: box.h } : { paths };
}

export function strokeShown(progress: number, stroke: number, strokes: number): number {
  return clamp(progress * strokes - stroke, 0, 1);
}
