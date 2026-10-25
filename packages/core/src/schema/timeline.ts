import { z } from "zod";
import { projectFontsSchema } from "#core/fonts/types";
import { isKeyOf, isRecord } from "#core/guards";
import { rectSchema, viewportSchema } from "#core/schema/actions";
import { recordingPropsSchema } from "#core/schema/recording";
import { DEFAULT_WORKFLOW_SETTINGS } from "#core/schema/workflow";

export const transitionRefSchema = z.object({
  kind: z.string(),
  durationInFrames: z.number().int().nonnegative(),
  params: z.record(z.string(), z.unknown()).optional(),
  hold: z.boolean().optional(),
});
export type TransitionRef = z.infer<typeof transitionRefSchema>;

export const EASES = ["linear", "in", "out", "in-out", "hold"] as const;
export type Ease = (typeof EASES)[number];

const keyframeSchema = z.object({
  frame: z.number().int().nonnegative(),
  value: z.number(),
  ease: z.enum(EASES).default("linear"),
});
export type Keyframe = z.infer<typeof keyframeSchema>;

const keyframes = () => z.array(keyframeSchema).optional();

export const visualEnvelopeSchema = z.object({
  opacity: keyframes(),
  x: keyframes(),
  y: keyframes(),
  scale: keyframes(),
});
export type VisualEnvelope = z.infer<typeof visualEnvelopeSchema>;
export const VISUAL_ENVELOPES = visualEnvelopeSchema.keyof().options;
export type VisualEnvelopeKey = keyof VisualEnvelope;

export const audioEnvelopeSchema = z.object({ volume: keyframes(), pan: keyframes() });
export type AudioEnvelope = z.infer<typeof audioEnvelopeSchema>;
export const AUDIO_ENVELOPES = audioEnvelopeSchema.keyof().options;
export type AudioEnvelopeKey = keyof AudioEnvelope;

export type EnvelopeKey = VisualEnvelopeKey | AudioEnvelopeKey;
export const cameraKeySchema = z.object({
  frame: z.number().int().nonnegative(),
  yaw: z.number().min(-180).max(180).default(0),
  pitch: z.number().min(-89).max(89).default(0),
  distance: z.number().min(0.3).max(4).default(1),
  height: z.number().min(-2).max(2).default(0),
  ease: z.enum(EASES).default("in-out"),
});
export type CameraKey = z.infer<typeof cameraKeySchema>;

export const cameraPathSchema = z.object({ keys: z.array(cameraKeySchema).default([]) });
export type CameraPath = z.infer<typeof cameraPathSchema>;

export const isCameraPath = (value: unknown): value is CameraPath =>
  cameraPathSchema.safeParse(value).success && isRecord(value) && Array.isArray(value.keys);

export const envelopeSchema = visualEnvelopeSchema.extend(audioEnvelopeSchema.shape);
export type Envelope = z.infer<typeof envelopeSchema>;

export const isEnvelope = (value: unknown): value is Envelope =>
  envelopeSchema.safeParse(value).success;

export const ENVELOPE_LABELS: Record<EnvelopeKey, string> = {
  opacity: "Opacity",
  x: "Move across",
  y: "Move up and down",
  scale: "Size",
  volume: "Volume",
  pan: "Balance",
};

export const isEnvelopeKey = (key: string): key is EnvelopeKey => isKeyOf(ENVELOPE_LABELS, key);

const PARAM_UNITS = ["ms", "frames", "rate", "number"] as const;

const paramRefSchema = z.object({
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

const editHandlesSchema = z.object({
  move: paramRefSchema.optional(),
  trimStart: paramRefSchema.optional(),
  trimEnd: paramRefSchema.optional(),
  speed: paramRefSchema.optional(),
  volume: paramRefSchema.optional(),
});
export type EditHandles = z.infer<typeof editHandlesSchema>;
export type EditHandleKind = keyof EditHandles;

const slotSchema = z.object({ nodeId: z.string(), port: z.string() });
export type Slot = z.infer<typeof slotSchema>;

export const elementRefSchema = z.object({
  component: z.string(),
  props: z.custom<Record<string, unknown>>(isRecord),
});

const remapSchema = z.array(z.tuple([z.number(), z.number()]));
export type Remap = z.infer<typeof remapSchema>;

const sequencerPropsSchema = elementRefSchema.extend({
  backgrounds: z.array(z.string().nullable()),
});
export type SequencerProps = z.infer<typeof sequencerPropsSchema>;

const groupPropsSchema = z.object({
  get tracks(): z.ZodArray<typeof timelineTrackSchema> {
    return z.array(timelineTrackSchema);
  },
  startFrom: z.number().optional(),
  remap: remapSchema.optional(),
  loopEvery: z.number().optional(),
  crop: rectSchema.optional(),
  wrapper: elementRefSchema.optional(),
  background: z.string().optional(),
  design: viewportSchema.optional(),
  sequencer: sequencerPropsSchema.optional(),
});
export type GroupProps = z.infer<typeof groupPropsSchema>;

const itemBaseSchema = z.object({
  id: z.string(),
  from: z.number().int(),
  durationInFrames: z.number().int().positive(),
  enter: transitionRefSchema.optional(),
  exit: transitionRefSchema.optional(),
  nodeId: z.string().optional(),
  handles: editHandlesSchema.optional(),
  slot: slotSchema.optional(),
  envelope: visualEnvelopeSchema.optional(),
  camera: cameraPathSchema.optional(),
  design: viewportSchema.extend({ exact: z.boolean().optional() }).optional(),
});
export type ItemBase = z.infer<typeof itemBaseSchema>;

const groupItemSchema = itemBaseSchema.extend({
  kind: z.literal("group"),
  props: groupPropsSchema,
});
export type GroupItem = z.infer<typeof groupItemSchema>;

const recordingItemSchema = itemBaseSchema.extend({
  kind: z.literal("recording"),
  props: recordingPropsSchema,
});
export type RecordingItem = z.infer<typeof recordingItemSchema>;

const elementItemSchema = itemBaseSchema.extend({
  kind: z.literal("element"),
  ...elementRefSchema.shape,
});
export type ElementItem = z.infer<typeof elementItemSchema>;

const timelineItemSchema = z.discriminatedUnion("kind", [
  groupItemSchema,
  recordingItemSchema,
  elementItemSchema,
]);

export type TimelineItem = z.infer<typeof timelineItemSchema>;

export const timelineTrackSchema = z.object({
  id: z.string(),
  items: z.array(timelineItemSchema),
  layer: z.boolean().optional(),
});
export type TimelineTrack = z.infer<typeof timelineTrackSchema>;

const duckSchema = z.object({
  level: z.number().min(0).max(1),
  fadeFrames: z.number().int().nonnegative(),
  keys: z.array(z.string()),
});

export const audioItemSchema = z.object({
  id: z.string(),
  src: z.string(),
  from: z.number().int(),
  durationInFrames: z.number().int().positive().optional(),
  volume: z.number().optional(),
  trimBefore: z.number().int().nonnegative().optional(),
  loop: z.boolean().optional(),
  rate: z.number().positive().optional(),
  fadeInFrames: z.number().int().nonnegative().optional(),
  fadeOutFrames: z.number().int().nonnegative().optional(),
  nodeId: z.string().optional(),
  channel: z.string().optional(),
  layerTrack: z.string().optional(),
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

const masterStemSchema = z.object({ src: z.string(), hash: z.string() });

export const REFRAMES = ["follow", "center", "fit"] as const;
export type ReframeMode = (typeof REFRAMES)[number];

export const PREVIEW_QUALITIES = ["fast", "exact"] as const;
export type PreviewQuality = (typeof PREVIEW_QUALITIES)[number];

export const timelineSpecSchema = z.object({
  width: z.number().int().positive(),
  height: z.number().int().positive(),
  fps: z.number().positive(),
  durationInFrames: z.number().int().positive(),
  background: z.string().optional(),
  tracks: z.array(timelineTrackSchema),
  audio: z.array(audioItemSchema).default([]),
  assets: z
    .object({
      projectBase: z.string().optional(),
      versions: z.record(z.string(), z.string()).optional(),
    })
    .optional(),
  reframe: z.enum(REFRAMES).optional(),
  quality: z.enum(PREVIEW_QUALITIES).optional(),
  fonts: projectFontsSchema.optional(),
  mix: masterStemSchema.optional(),
});
export type TimelineSpec = z.infer<typeof timelineSpecSchema>;

export const EMPTY_SPEC: TimelineSpec = {
  width: DEFAULT_WORKFLOW_SETTINGS.width,
  height: DEFAULT_WORKFLOW_SETTINGS.height,
  fps: DEFAULT_WORKFLOW_SETTINGS.fps,
  durationInFrames: 90,
  tracks: [],
  audio: [],
};
