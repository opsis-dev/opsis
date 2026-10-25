import { z } from "zod";
import { lerp } from "../math";

export const rectSchema = z.object({ x: z.number(), y: z.number(), w: z.number(), h: z.number() });
export type Rect = z.infer<typeof rectSchema>;

const pointSchema = z.object({ x: z.number(), y: z.number() });
export type Point = z.infer<typeof pointSchema>;

export const viewportSchema = z.object({ width: z.number(), height: z.number() });
export type Viewport = z.infer<typeof viewportSchema>;

export const boundsSchema = z.object({
  x: z.number(),
  y: z.number(),
  width: z.number(),
  height: z.number(),
});
export type Bounds = z.infer<typeof boundsSchema>;

const POINT_TEXT = /^\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*$/;

export function parsePoint(text: string): Point | null {
  const m = POINT_TEXT.exec(text);
  return m ? { x: Number(m[1]), y: Number(m[2]) } : null;
}

export type PathText = { ok: true; points: [Point, ...Point[]] } | { ok: false; problem: string };

export function readPath(text: string): PathText {
  const points: Point[] = [];
  for (const [i, line] of text.split("\n").entries()) {
    if (!line.trim()) continue;
    const point = parsePoint(line);
    if (!point) return { ok: false, problem: `Line ${i + 1} of the path isn't two numbers` };
    points.push(point);
  }
  const [first, ...rest] = points;
  if (!first) return { ok: false, problem: "Add points to the path" };
  if (points.every((p) => p.x === 0 && p.y === 0))
    return { ok: false, problem: "Add a point away from where the drag starts" };
  return { ok: true, points: [first, ...rest] };
}

export const isBoundsList = (value: unknown): value is Bounds[] =>
  z.array(boundsSchema).safeParse(value).success;

export const rectFromBounds = (b: Bounds): Rect => ({ x: b.x, y: b.y, w: b.width, h: b.height });

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

const ACTION_KINDS = [
  "click",
  "type",
  "hover",
  "scroll",
  "navigate",
  "wait",
  "chapter",
  "key",
  "focus",
  "drag",
] as const;

const actionSchema = z.object({
  id: z.string(),
  kind: z.enum(ACTION_KINDS),
  t: z.number(),
  tEnd: z.number().optional(),
  tDepart: z.number().optional(),
  tDown: z.number().optional(),
  point: pointSchema.optional(),
  rect: rectSchema.optional(),
  cluster: z.string().optional(),
  label: z.string().optional(),
  selectorUsed: z.string().optional(),
  zoom: z.boolean().optional(),
  zoomScale: z.number().optional(),
  text: z.string().optional(),
  nodeId: z.string().optional(),
  path: z.array(pointSchema).max(64).optional(),
  drop: rectSchema.optional(),
});
export type Action = z.infer<typeof actionSchema>;

const cursorSampleSchema = z.object({
  t: z.number(),
  x: z.number(),
  y: z.number(),
  c: z.string().optional(),
});

export const actionLogSchema = z.object({
  viewport: viewportSchema,
  dpr: z.number(),
  t0: z.number(),
  durationMs: z.number(),
  actions: z.array(actionSchema),
  cursorTrack: z.array(cursorSampleSchema).optional(),
});
export type ActionLog = z.infer<typeof actionLogSchema>;
