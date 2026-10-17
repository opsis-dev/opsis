import { z } from "zod";
import { isRecord } from "../guards";
import type { ActionLog } from "./actions";
import type { AudioItem, TimelineSpec, TimelineTrack, TransitionRef } from "./timeline";

export type AssetRef = string;

export type RecordingValue = {
  type: "recording";
  src: AssetRef | null;
  proxySrc?: AssetRef;
  log: ActionLog | null;
  durationMs: number;
  viewport: { width: number; height: number };
  nodeId: string;
};

type ImageValue = {
  type: "image";
  src: AssetRef | null;
  width?: number;
  height?: number;
  scale?: number;
  body?: { x: number; y: number; width: number; height: number };
};

export type MediaValue = {
  type: "media";
  src: AssetRef | null;
  kind: "video" | "audio" | "image";
  durationMs?: number;
  width?: number;
  height?: number;
  hasAudio?: boolean;
};

type FileValue = { type: "file"; src: AssetRef | null };

export type ClipValue = {
  type: "clip";
  durationInFrames: number;
  tracks: TimelineTrack[];
  audio: AudioItem[];
  recording?: RecordingValue;
};

type AudioValue = { type: "audio"; items: AudioItem[]; durationInFrames?: number };

export type CaptionsValue = {
  type: "captions";
  captions: Array<{ text: string; startMs: number; endMs: number }>;
};

export type TimelineValue = { type: "timeline"; spec: TimelineSpec };

type TransitionValue = { type: "transition"; ref: TransitionRef };

type SessionValue = { type: "session" };

export type CompileValue =
  | RecordingValue
  | ImageValue
  | MediaValue
  | FileValue
  | ClipValue
  | AudioValue
  | CaptionsValue
  | TimelineValue
  | TransitionValue
  | SessionValue;

const VALUE_TYPES: Record<CompileValue["type"], true> = {
  recording: true,
  image: true,
  media: true,
  file: true,
  clip: true,
  audio: true,
  captions: true,
  timeline: true,
  transition: true,
  session: true,
};

const compileValueSchema = z.custom<CompileValue>(
  (v) => isRecord(v) && typeof v.type === "string" && Object.hasOwn(VALUE_TYPES, v.type),
);

const artifactSchema = z.object({
  kind: z.enum(["video", "image", "audio", "json", "file"]),
  path: z.string(),
  label: z.string().optional(),
  poster: z.string().optional(),
  bytes: z.number().optional(),
});
export type Artifact = z.infer<typeof artifactSchema>;

export const nodeResultSchema = z.object({
  hash: z.string(),
  outputs: z.record(z.string(), compileValueSchema),
  artifacts: z.array(artifactSchema),
  finishedAt: z.number(),
  durationMs: z.number(),
  traceId: z.string().optional(),
});
export type NodeResult = z.infer<typeof nodeResultSchema>;

const nodeFailureSchema = z.object({
  message: z.string(),
  at: z.number(),
  traceId: z.string().optional(),
});
export type NodeFailure = z.infer<typeof nodeFailureSchema>;

export const lastRunSchema = z.object({
  updatedAt: z.number(),
  nodes: z.record(z.string(), nodeResultSchema),
  failures: z.record(z.string(), nodeFailureSchema).optional(),
});
export type LastRun = z.infer<typeof lastRunSchema>;

type ValueOf<K extends CompileValue["type"]> = Extract<CompileValue, { type: K }>;

const isValue = <K extends CompileValue["type"]>(
  value: CompileValue | undefined,
  type: K,
): value is ValueOf<K> => value?.type === type;

export const asValue = <K extends CompileValue["type"]>(
  value: CompileValue | undefined,
  type: K,
): ValueOf<K> | undefined => (isValue(value, type) ? value : undefined);

export const outputOf = <K extends CompileValue["type"]>(
  result: NodeResult | undefined,
  port: string,
  type: K,
): ValueOf<K> | undefined => asValue(result?.outputs[port], type);
