import { z } from "zod";
import { actionLogSchema, boundsSchema, viewportSchema } from "./actions";
import { audioItemSchema, timelineTrackSchema } from "./timeline";

const assetRefSchema = z.string().nullable();

const paramsSchema = z.record(z.string(), z.unknown());

const styleEntrySchema = z.object({ nodeId: z.string(), type: z.string(), params: paramsSchema });
export type StyleEntry = z.infer<typeof styleEntrySchema>;

const recordingValueSchema = z.object({
  type: z.literal("recording"),
  src: assetRefSchema,
  proxySrc: z.string().optional(),
  log: actionLogSchema.nullable(),
  durationMs: z.number(),
  viewport: viewportSchema,
  nodeId: z.string(),
  style: z.array(styleEntrySchema),
});
export type RecordingValue = z.infer<typeof recordingValueSchema>;

const imageValueSchema = z.object({
  type: z.literal("image"),
  src: assetRefSchema,
  width: z.number().optional(),
  height: z.number().optional(),
  scale: z.number().optional(),
  body: boundsSchema.optional(),
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

const fileInfoValueSchema = z.object({ type: z.literal("file-info"), src: assetRefSchema });

const folderValueSchema = z.object({ type: z.literal("folder"), src: assetRefSchema });

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

const sessionStepSchema = z.object({
  id: z.string(),
  type: z.string(),
  label: z.string().optional(),
  params: paramsSchema,
});
export type SessionStep = z.infer<typeof sessionStepSchema>;

const browserSessionValueSchema = z.object({
  type: z.literal("browser-session"),
  browser: z.object({ nodeId: z.string(), params: paramsSchema }),
  steps: z.array(sessionStepSchema),
});
export type BrowserSessionValue = z.infer<typeof browserSessionValueSchema>;

const recordingStyleValueSchema = z.object({
  type: z.literal("recording-style"),
  entries: z.array(styleEntrySchema),
});
export type RecordingStyleValue = z.infer<typeof recordingStyleValueSchema>;

const compileValueSchema = z.discriminatedUnion("type", [
  browserSessionValueSchema,
  recordingValueSchema,
  recordingStyleValueSchema,
  imageValueSchema,
  mediaValueSchema,
  fileInfoValueSchema,
  folderValueSchema,
  clipValueSchema,
  audioValueSchema,
  captionsValueSchema,
]);
export type CompileValue = z.infer<typeof compileValueSchema>;

const artifactSchema = z.object({
  kind: z.enum(["video", "image", "audio", "json", "file"]),
  path: z.string(),
  label: z.string().optional(),
  poster: z.string().optional(),
  bytes: z.number().optional(),
  stamp: z.string().optional(),
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
