import { z } from "zod";
import { actionLogSchema, viewportSchema } from "./actions";
import {
  audioItemSchema,
  timelineSpecSchema,
  timelineTrackSchema,
  transitionRefSchema,
} from "./timeline";

const assetRefSchema = z.string().nullable();

const recordingValueSchema = z.object({
  type: z.literal("recording"),
  src: assetRefSchema,
  proxySrc: z.string().optional(),
  log: actionLogSchema.nullable(),
  durationMs: z.number(),
  viewport: viewportSchema,
  nodeId: z.string(),
});
export type RecordingValue = z.infer<typeof recordingValueSchema>;

const imageValueSchema = z.object({
  type: z.literal("image"),
  src: assetRefSchema,
  width: z.number().optional(),
  height: z.number().optional(),
  scale: z.number().optional(),
  body: z
    .object({ x: z.number(), y: z.number(), width: z.number(), height: z.number() })
    .optional(),
});

const mediaValueSchema = z.object({
  type: z.literal("media"),
  src: assetRefSchema,
  kind: z.enum(["video", "audio", "image"]),
  durationMs: z.number().optional(),
  width: z.number().optional(),
  height: z.number().optional(),
  hasAudio: z.boolean().optional(),
});
export type MediaValue = z.infer<typeof mediaValueSchema>;

const fileValueSchema = z.object({ type: z.literal("file"), src: assetRefSchema });

const clipValueSchema = z.object({
  type: z.literal("clip"),
  durationInFrames: z.number(),
  tracks: z.array(timelineTrackSchema),
  audio: z.array(audioItemSchema),
  recording: recordingValueSchema.optional(),
});
export type ClipValue = z.infer<typeof clipValueSchema>;

const audioValueSchema = z.object({
  type: z.literal("audio"),
  items: z.array(audioItemSchema),
  durationInFrames: z.number().optional(),
});

const captionsValueSchema = z.object({
  type: z.literal("captions"),
  captions: z.array(z.object({ text: z.string(), startMs: z.number(), endMs: z.number() })),
});
export type CaptionsValue = z.infer<typeof captionsValueSchema>;

const timelineValueSchema = z.object({ type: z.literal("timeline"), spec: timelineSpecSchema });
export type TimelineValue = z.infer<typeof timelineValueSchema>;

const transitionValueSchema = z.object({ type: z.literal("transition"), ref: transitionRefSchema });

const sessionValueSchema = z.object({ type: z.literal("session") });

const compileValueSchema = z.discriminatedUnion("type", [
  recordingValueSchema,
  imageValueSchema,
  mediaValueSchema,
  fileValueSchema,
  clipValueSchema,
  audioValueSchema,
  captionsValueSchema,
  timelineValueSchema,
  transitionValueSchema,
  sessionValueSchema,
]);
export type CompileValue = z.infer<typeof compileValueSchema>;

const artifactSchema = z.object({
  kind: z.enum(["video", "image", "audio", "json", "file"]),
  path: z.string(),
  label: z.string().optional(),
  poster: z.string().optional(),
  bytes: z.number().optional(),
});
export type Artifact = z.infer<typeof artifactSchema>;

export const artifactName = (artifact: Artifact): string => artifact.label ?? artifact.path;

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

const validEntries = <T>(schema: z.ZodType<T>) =>
  z.record(z.string(), z.unknown()).transform((entries) =>
    Object.fromEntries(
      Object.entries(entries).flatMap(([key, value]) => {
        const parsed = schema.safeParse(value);
        return parsed.success ? [[key, parsed.data] as const] : [];
      }),
    ),
  );

export const lastRunSchema = z.object({
  updatedAt: z.number(),
  nodes: validEntries(nodeResultSchema),
  failures: validEntries(nodeFailureSchema).optional(),
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
