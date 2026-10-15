import type { z } from "zod";
import type { CompileValue, NodeResult } from "../schema/values";
import type { CardContext, CardRow } from "./card";
import type { PortType, Workflow, WorkflowSettings } from "../schema/workflow";

export type NodeCategory =
  | "Browser"
  | "Capture"
  | "Camera"
  | "Cursor"
  | "Text & overlays"
  | "Transitions"
  | "Backgrounds"
  | "Effects"
  | "Elements"
  | "Audio"
  | "Timeline"
  | "Media"
  | "Output"
  | "Utility";

export const NODE_CATEGORIES: NodeCategory[] = [
  "Browser",
  "Capture",
  "Camera",
  "Cursor",
  "Text & overlays",
  "Transitions",
  "Backgrounds",
  "Effects",
  "Elements",
  "Audio",
  "Timeline",
  "Media",
  "Output",
  "Utility",
];

export const CATEGORY_LABELS: Record<NodeCategory, string> = {
  Browser: "Browser",
  Capture: "Record",
  Camera: "Camera",
  Cursor: "Cursor & clicks",
  "Text & overlays": "Text & overlays",
  Transitions: "Transitions",
  Backgrounds: "Backgrounds",
  Effects: "Effects",
  Elements: "Graphics",
  Audio: "Sound",
  Timeline: "Timeline",
  Media: "Media",
  Output: "Export",
  Utility: "Organise",
};

export type NodePhase = "capture" | "media" | "compile" | "render" | "post";

export type PortSpec = { type: PortType; label?: string; optional?: boolean };

export type CompileContext<P> = {
  nodeId: string;
  params: P;
  inputs: Partial<Record<string, CompileValue>>;
  settings: WorkflowSettings;
  result?: NodeResult;
  vars: Record<string, string>;
  workflow: Workflow;
};

export type CompileOutputs = Partial<Record<string, CompileValue>>;

// oxlint-disable-next-line typescript/no-explicit-any
export interface NodeDef<S extends z.ZodObject<any> = z.ZodObject<any>> {
  type: string;
  title: string;
  category: NodeCategory;
  description?: string;
  icon?: string;
  phase: NodePhase;
  inputs: Record<string, PortSpec>;
  outputs: Record<string, PortSpec>;
  params: S;
  virtual?: boolean;
  executes?: boolean;
  volatile?: boolean;
  mutates?: (params: z.infer<S>) => string | undefined;
  element?: string;
  hidden?: boolean;
  compile?: (ctx: CompileContext<z.infer<S>>) => CompileOutputs | undefined;
  card?: (ctx: CardContext<z.infer<S>>) => Array<CardRow | false | null | undefined | "" | 0>;
}

// oxlint-disable-next-line typescript/no-explicit-any
export function defineNode<S extends z.ZodObject<any>>(def: NodeDef<S>): NodeDef<S> {
  return def;
}

export const port = (type: PortType, label?: string, optional = false): PortSpec => ({
  type,
  label,
  optional,
});

export const opt = (type: PortType, label?: string): PortSpec => port(type, label, true);
