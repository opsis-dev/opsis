import type { z } from "zod";
import { isClip } from "../compile/clip";
import type { ClipValue, CompileValue, NodeResult } from "../schema/values";
import type { CardContext, CardRow } from "./card";
import type { NodeIconName } from "./icon-name";
import type { PortType, Workflow, WorkflowNode, WorkflowSettings } from "../schema/workflow";
import { interpolate } from "../vars";
import { withDefaults } from "../elements/introspect";

export const NODE_CATEGORIES = [
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
] as const;

export type NodeCategory = (typeof NODE_CATEGORIES)[number];

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

export type PortSpec = { type: PortType; label?: string; optional?: boolean; many?: boolean };

export type CompileInputs = Partial<Record<string, CompileValue>>;

type CompileContext<P> = {
  nodeId: string;
  params: P;
  inputs: CompileInputs;
  settings: WorkflowSettings;
  result?: NodeResult;
  vars: Record<string, string>;
  workflow: Workflow;
};

export type CompileOutputs = Partial<Record<string, CompileValue>>;

export interface NodeDef<S extends z.ZodObject = z.ZodObject> {
  type: string;
  title: string;
  category: NodeCategory;
  description?: string;
  icon?: NodeIconName;
  phase: NodePhase;
  inputs: Record<string, PortSpec>;
  outputs: Record<string, PortSpec>;
  params: S;
  virtual?: boolean;
  action?: boolean;
  executes?: boolean;
  volatile?: boolean;
  mutates?(params: z.infer<S>): string | undefined;
  element?: string;
  hidden?: boolean;
  cardHeight?: number;
  hostDefaults?(platform: NodeJS.Platform): Partial<z.infer<S>>;
  compile?(ctx: CompileContext<z.infer<S>>): CompileOutputs | undefined;
  card?(ctx: CardContext<z.infer<S>>): Array<CardRow | false | null | undefined | "" | 0>;
}

export type NodeParams<D extends NodeDef> = z.infer<D["params"]>;

export const hasNoCardPicture = (def: Pick<NodeDef, "category">): boolean =>
  def.category === "Browser" || def.category === "Utility";

export function defineNode<S extends z.ZodObject>(def: NodeDef<S>): NodeDef<S> {
  return def;
}

export const interpolatedParams = (
  def: Pick<NodeDef, "params"> | undefined,
  params: WorkflowNode["params"],
  vars: Record<string, string>,
): WorkflowNode["params"] => interpolate(def ? withDefaults(def.params, params) : params, vars);

export const parseNodeParams = <S extends z.ZodObject>(
  def: NodeDef<S>,
  node: Pick<WorkflowNode, "params">,
  vars: Record<string, string>,
): z.infer<S> => def.params.parse(interpolatedParams(def, node.params, vars));

export const port = (type: PortType, label?: string): PortSpec => ({
  type,
  label,
  optional: false,
});

export const opt = (type: PortType, label?: string): PortSpec => ({
  ...port(type, label),
  optional: true,
});

export const many = (type: PortType, label?: string): PortSpec => ({
  ...opt(type, label),
  many: true,
});

export const clipModifier = {
  inputs: { clip: port("clip", "Clip") },
  outputs: { clip: port("clip", "Clip") },
} as const;

export const clipInput = (inputs: CompileInputs, name = "clip"): ClipValue | undefined => {
  const value = inputs[name];
  return isClip(value) ? value : undefined;
};

export const withClip = (
  inputs: CompileInputs,
  fn: (clip: ClipValue) => ClipValue,
): CompileOutputs => {
  const clip = clipInput(inputs);
  return clip ? { clip: fn(clip) } : {};
};
