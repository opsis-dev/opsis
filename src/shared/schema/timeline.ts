import { z } from "zod";
import { projectFontsSchema } from "../fonts/types";

export const transitionRefSchema = z.object({
  kind: z.string(),
  durationInFrames: z.number().int().nonnegative(),
  params: z.record(z.string(), z.unknown()).optional(),
  hold: z.boolean().optional(),
});
export type TransitionRef = z.infer<typeof transitionRefSchema>;

export const EASES = ["linear", "in", "out", "in-out", "hold"] as const;
export type Ease = (typeof EASES)[number];

export const keyframeSchema = z.object({
  frame: z.number().int().nonnegative(),
  value: z.number(),
  ease: z.enum(EASES).default("linear"),
});
export type Keyframe = z.infer<typeof keyframeSchema>;

export const VISUAL_ENVELOPES = ["opacity", "x", "y", "scale"] as const;
export const AUDIO_ENVELOPES = ["volume", "pan"] as const;
export type VisualEnvelopeKey = (typeof VISUAL_ENVELOPES)[number];
export type AudioEnvelopeKey = (typeof AUDIO_ENVELOPES)[number];

export const visualEnvelopeSchema = z.object({
  opacity: z.array(keyframeSchema).optional(),
  x: z.array(keyframeSchema).optional(),
  y: z.array(keyframeSchema).optional(),
  scale: z.array(keyframeSchema).optional(),
});
export type VisualEnvelope = z.infer<typeof visualEnvelopeSchema>;

export const audioEnvelopeSchema = z.object({
  volume: z.array(keyframeSchema).optional(),
  pan: z.array(keyframeSchema).optional(),
});
export type AudioEnvelope = z.infer<typeof audioEnvelopeSchema>;

export const PARAM_UNITS = ["ms", "frames", "rate", "number"] as const;

export const paramRefSchema = z.object({
  nodeId: z.string(),
  param: z.string(),
  unit: z.enum(PARAM_UNITS),
  scale: z.number().optional(),
  index: z.number().int().nonnegative().optional(),
  linked: z
    .array(
      z.object({
        param: z.string(),
        sign: z.union([z.literal(1), z.literal(-1)]),
        ifNonZero: z.boolean().optional(),
      }),
    )
    .optional(),
});
export type ParamRef = z.infer<typeof paramRefSchema>;

export const editHandlesSchema = z.object({
  move: paramRefSchema.optional(),
  trimStart: paramRefSchema.optional(),
  trimEnd: paramRefSchema.optional(),
  speed: paramRefSchema.optional(),
  volume: paramRefSchema.optional(),
});
export type EditHandles = z.infer<typeof editHandlesSchema>;
export type EditHandleKind = keyof EditHandles;

export const slotSchema = z.object({ nodeId: z.string(), port: z.string() });
export type Slot = z.infer<typeof slotSchema>;

export type TimelineItem = {
  id: string;
  from: number;
  durationInFrames: number;
  component: string;
  props: Record<string, unknown>;
  enter?: TransitionRef;
  exit?: TransitionRef;
  nodeId?: string;
  handles?: EditHandles;
  slot?: Slot;
  envelope?: VisualEnvelope;
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
  handles: editHandlesSchema.optional(),
  slot: slotSchema.optional(),
  envelope: visualEnvelopeSchema.optional(),
});

export const timelineTrackSchema: z.ZodType<TimelineTrack> = z.object({
  id: z.string(),
  items: z.array(timelineItemSchema),
});

export const duckSchema = z.object({
  level: z.number().min(0).max(1),
  fadeFrames: z.number().int().nonnegative(),
  keys: z.array(z.string()),
});
export type Duck = z.infer<typeof duckSchema>;

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
  channel: z.string().optional(),
  role: z.enum(["music", "effect"]).optional(),
  lengthFrames: z.number().int().positive().optional(),
  keepMusic: z.boolean().optional(),
  gain: z.number().nonnegative().optional(),
  duck: duckSchema.optional(),
  envelope: audioEnvelopeSchema.optional(),
  handles: editHandlesSchema.optional(),
  anchor: z.object({ nodeId: z.string() }).optional(),
});
export type AudioItem = z.infer<typeof audioItemSchema>;

export const masterStemSchema = z.object({ src: z.string(), hash: z.string() });
export type MasterStem = z.infer<typeof masterStemSchema>;

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
  fonts: projectFontsSchema.optional(),
  mix: masterStemSchema.optional(),
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
