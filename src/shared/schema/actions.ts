import { z } from "zod";

const rectSchema = z.object({ x: z.number(), y: z.number(), w: z.number(), h: z.number() });
export type Rect = z.infer<typeof rectSchema>;

export type Bounds = { x: number; y: number; width: number; height: number };

export const rectFromBounds = (b: Bounds): Rect => ({ x: b.x, y: b.y, w: b.width, h: b.height });

export const boundsFromRect = (r: Rect): Bounds => ({ x: r.x, y: r.y, width: r.w, height: r.h });

export const rectCentre = (r: Rect): { x: number; y: number } => ({
  x: r.x + r.w / 2,
  y: r.y + r.h / 2,
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

const pointSchema = z.object({ x: z.number(), y: z.number() });

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

const cursorSampleSchema = z.object({ t: z.number(), x: z.number(), y: z.number() });

const actionLogSchema = z.object({
  viewport: z.object({ width: z.number(), height: z.number() }),
  dpr: z.number(),
  t0: z.number(),
  durationMs: z.number(),
  actions: z.array(actionSchema),
  cursorTrack: z.array(cursorSampleSchema).optional(),
});
export type ActionLog = z.infer<typeof actionLogSchema>;
