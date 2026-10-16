import { z } from "zod";

export const rectSchema = z.object({ x: z.number(), y: z.number(), w: z.number(), h: z.number() });
export type Rect = z.infer<typeof rectSchema>;

export const ACTION_KINDS = [
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

export const pointSchema = z.object({ x: z.number(), y: z.number() });

export const actionSchema = z.object({
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

export const cursorSampleSchema = z.object({ t: z.number(), x: z.number(), y: z.number() });

export const actionLogSchema = z.object({
  viewport: z.object({ width: z.number(), height: z.number() }),
  dpr: z.number(),
  t0: z.number(),
  durationMs: z.number(),
  actions: z.array(actionSchema),
  cursorTrack: z.array(cursorSampleSchema).optional(),
});
export type ActionLog = z.infer<typeof actionLogSchema>;
