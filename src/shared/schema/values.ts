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

export type ImageValue = {
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

export type AudioValue = { type: "audio"; items: AudioItem[]; durationInFrames?: number };

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

export type Artifact = {
  kind: "video" | "image" | "audio" | "json" | "file";
  path: string;
  label?: string;
  poster?: string;
  bytes?: number;
};

export type NodeResult = {
  hash: string;
  outputs: Record<string, CompileValue>;
  artifacts: Artifact[];
  finishedAt: number;
  durationMs: number;
  traceId?: string;
};

export type NodeFailure = { message: string; at: number; traceId?: string };

export type LastRun = {
  updatedAt: number;
  nodes: Record<string, NodeResult>;
  failures?: Record<string, NodeFailure>;
};

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
