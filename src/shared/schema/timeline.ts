import { z } from "zod";

export const transitionRefSchema = z.object({
  kind: z.string(),
  durationInFrames: z.number().int().nonnegative(),
  params: z.record(z.string(), z.unknown()).optional(),
});
export type TransitionRef = z.infer<typeof transitionRefSchema>;

export type TimelineItem = {
  id: string;
  from: number;
  durationInFrames: number;
  component: string;
  props: Record<string, unknown>;
  enter?: TransitionRef;
  exit?: TransitionRef;
  nodeId?: string;
};

export type TimelineTrack = { id: string; items: TimelineItem[] };

export const timelineItemSchema: z.ZodType<TimelineItem> = z.object({
  id: z.string(),
  from: z.number().int(),
  durationInFrames: z.number().int().positive(),
  component: z.string(),
  props: z.record(z.string(), z.unknown()),
  enter: transitionRefSchema.optional(),
  exit: transitionRefSchema.optional(),
  nodeId: z.string().optional(),
});

export const timelineTrackSchema: z.ZodType<TimelineTrack> = z.object({
  id: z.string(),
  items: z.array(timelineItemSchema),
});

export const audioItemSchema = z.object({
  id: z.string(),
  src: z.string(),
  from: z.number().int(),
  durationInFrames: z.number().int().positive().optional(),
  volume: z.union([z.number(), z.string()]).optional(),
  trimBefore: z.number().int().nonnegative().optional(),
  loop: z.boolean().optional(),
  fadeInFrames: z.number().int().nonnegative().optional(),
  fadeOutFrames: z.number().int().nonnegative().optional(),
  nodeId: z.string().optional(),
});
export type AudioItem = z.infer<typeof audioItemSchema>;

export const timelineSpecSchema = z.object({
  width: z.number().int().positive(),
  height: z.number().int().positive(),
  fps: z.number().positive(),
  durationInFrames: z.number().int().positive(),
  background: z.string().optional(),
  tracks: z.array(timelineTrackSchema),
  audio: z.array(audioItemSchema).default([]),
  assets: z.object({ projectBase: z.string().optional() }).optional(),
  reframe: z.enum(["follow", "center", "fit"]).optional(),
  quality: z.enum(["fast", "exact"]).optional(),
});
export type TimelineSpec = z.infer<typeof timelineSpecSchema>;

export const EMPTY_SPEC: TimelineSpec = {
  width: 1920,
  height: 1080,
  fps: 30,
  durationInFrames: 90,
  tracks: [],
  audio: [],
};
