import { z } from "zod";
import { DEFAULT_PROJECT_FONTS, projectFontsSchema } from "../fonts/types";
import { mixSchema } from "../mix/types";

const PORT_TYPES = [
  "session",
  "recording",
  "clip",
  "scene",
  "audio",
  "captions",
  "image",
  "media",
  "timeline",
  "file",
] as const;
export type PortType = (typeof PORT_TYPES)[number];

export const PORT_WORDS: Record<PortType, string> = {
  session: "browser action",
  recording: "recording",
  clip: "clip",
  scene: "clip",
  audio: "sound",
  captions: "captions",
  image: "image",
  media: "file",
  timeline: "clip",
  file: "file",
};

export const workflowSettingsSchema = z.object({
  width: z.number().int().min(16).max(7680).default(1920),
  height: z.number().int().min(16).max(7680).default(1080),
  fps: z.union([z.literal(30), z.literal(60)]).default(30),
  background: z.string().optional(),
  theme: z.enum(["light", "dark"]).default("dark"),
  fonts: projectFontsSchema.default(DEFAULT_PROJECT_FONTS),
});
export type WorkflowSettings = z.infer<typeof workflowSettingsSchema>;

export const DEFAULT_WORKFLOW_SETTINGS: WorkflowSettings = workflowSettingsSchema.parse({});

export const workflowNodeSchema = z.object({
  id: z.string().min(1),
  type: z.string().min(1),
  position: z.object({ x: z.number(), y: z.number() }),
  params: z.record(z.string(), z.unknown()).default({}),
  label: z.string().optional(),
  disabled: z.boolean().optional(),
  parentId: z.string().optional(),
  size: z.object({ width: z.number(), height: z.number() }).optional(),
});
export type WorkflowNode = z.infer<typeof workflowNodeSchema>;

export const workflowEdgeSchema = z.object({
  id: z.string().min(1),
  source: z.string(),
  sourceHandle: z.string(),
  target: z.string(),
  targetHandle: z.string(),
});
export type WorkflowEdge = z.infer<typeof workflowEdgeSchema>;

export type EdgeEnds = Omit<WorkflowEdge, "id">;

const markerSchema = z.object({
  id: z.string().min(1),
  atMs: z.number().int().nonnegative(),
  label: z.string().default(""),
});
export type Marker = z.infer<typeof markerSchema>;

export const workflowSchema = z.object({
  version: z.literal(1),
  id: z.string(),
  name: z.string(),
  settings: workflowSettingsSchema.default(DEFAULT_WORKFLOW_SETTINGS),
  variables: z.record(z.string(), z.string()).default({}),
  nodes: z.array(workflowNodeSchema),
  edges: z.array(workflowEdgeSchema),
  mix: mixSchema.optional(),
  markers: z.array(markerSchema).default([]),
});
export type Workflow = z.infer<typeof workflowSchema>;

export const WORKFLOW_FILE = "project.demoflow.json";

export function normalizeWorkflow(wf: Workflow): Workflow {
  if (!wf.mix) return wf;
  const ids = new Set(wf.nodes.map((n) => n.id));
  const channels = Object.entries(wf.mix.channels).filter(([id]) => ids.has(id));
  return { ...wf, mix: { ...wf.mix, channels: Object.fromEntries(channels) } };
}
