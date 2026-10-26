import { isKeyOf, isRecord } from "@opsis/video/common/guards";
import { z } from "zod";
import type {
  AudioEnvelope,
  CameraPath,
  Ease,
  Keyframe,
  PreviewQuality,
  VisualEnvelope,
} from "@/generated/bindings";

export const EASES: Ease[] = ["linear", "in", "out", "in-out", "hold"];

export const VISUAL_ENVELOPES = [
  "opacity",
  "x",
  "y",
  "scale",
] as const satisfies (keyof VisualEnvelope)[];
export type VisualEnvelopeKey = keyof VisualEnvelope;

export const AUDIO_ENVELOPES = ["volume", "pan"] as const satisfies (keyof AudioEnvelope)[];
export type AudioEnvelopeKey = keyof AudioEnvelope;

export type EnvelopeKey = VisualEnvelopeKey | AudioEnvelopeKey;

export type Envelope = Partial<Record<EnvelopeKey, Keyframe[]>>;

export const PREVIEW_QUALITIES: PreviewQuality[] = ["fast", "exact"];

const keyframeSchema = z.object({
  frame: z.number().int().nonnegative(),
  value: z.number(),
  ease: z.enum(EASES).default("linear"),
});

const keyframes = () => z.array(keyframeSchema).optional();

export const envelopeSchema = z.object({
  opacity: keyframes(),
  x: keyframes(),
  y: keyframes(),
  scale: keyframes(),
  volume: keyframes(),
  pan: keyframes(),
});

const cameraKeySchema = z.object({
  frame: z.number().int().nonnegative(),
  yaw: z.number().min(-180).max(180).default(0),
  pitch: z.number().min(-89).max(89).default(0),
  distance: z.number().min(0.3).max(4).default(1),
  height: z.number().min(-2).max(2).default(0),
  ease: z.enum(EASES).default("in-out"),
});

export const cameraPathSchema = z.object({ keys: z.array(cameraKeySchema).default([]) });

export const isCameraPath = (value: unknown): value is CameraPath =>
  cameraPathSchema.safeParse(value).success && isRecord(value) && Array.isArray(value.keys);

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
