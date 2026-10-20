import type { z } from "zod";
import { isClip } from "../compile/clip";
import { schemaDefaults, withDefaults } from "../elements/introspect";
import type { ElementMeta } from "../elements/types";
import type { ClipValue, CompileValue, NodeResult } from "../schema/values";
import type { PortType, Workflow, WorkflowNode, WorkflowSettings } from "../schema/workflow";
import { interpolate, namesForUnknownVariables } from "../vars";
import { MAX_CARD_ROWS, sentence, type CardContext, type CardRow } from "./card";

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

export type CardLine = CardRow | false | null | undefined | "" | 0;

export interface NodeDef<S extends z.ZodObject = z.ZodObject> {
  type: string;
  title: string;
  category: NodeCategory;
  description?: string;
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
  card?(ctx: CardContext<z.infer<S>>): CardLine[];
}

export type NodeParams<D extends NodeDef> = z.infer<D["params"]>;

const hasNoCardPicture = (def: Pick<NodeDef, "category">): boolean =>
  def.category === "Browser" || def.category === "Utility";

export const showsClipStill = (def: Pick<NodeDef, "category" | "outputs">): boolean =>
  !hasNoCardPicture(def) && CLIP_PORT in def.outputs;

export const hasCardPicture = (def: NodeDef): boolean =>
  showsClipStill(def) || (!hasNoCardPicture(def) && !!def.executes && def.phase !== "post");

export const cardRows = <S extends z.ZodObject>(
  def: NodeDef<S>,
  ctx: CardContext<z.infer<S>>,
): CardRow[] => (def.card?.(ctx) ?? []).filter((r): r is CardRow => !!r).slice(0, MAX_CARD_ROWS);

export function defineNode<S extends z.ZodObject>(def: NodeDef<S>): NodeDef<S> {
  return def;
}

export const interpolatedParams = (
  def: Pick<NodeDef, "params"> | undefined,
  params: WorkflowNode["params"],
  vars: Record<string, string>,
): WorkflowNode["params"] => interpolate(def ? withDefaults(def.params, params) : params, vars);

function sameKind(a: unknown, b: unknown): boolean {
  return typeof a === typeof b && Array.isArray(a) === Array.isArray(b);
}

export function cardParams(
  def: Pick<NodeDef, "params">,
  params: WorkflowNode["params"],
  vars: Record<string, string>,
): WorkflowNode["params"] {
  const values = namesForUnknownVariables(interpolatedParams(def, params, vars));
  const defaults = schemaDefaults(def.params);
  for (const [key, fallback] of Object.entries(defaults))
    if (!sameKind(values[key], fallback)) values[key] = fallback;
  return values;
}

export const parseNodeParams = <S extends z.ZodObject>(
  def: NodeDef<S>,
  node: Pick<WorkflowNode, "params">,
  vars: Record<string, string>,
): z.infer<S> => def.params.parse(interpolatedParams(def, node.params, vars));

export const port = (type: PortType, label?: string): PortSpec => ({ type, label });

export const opt = (type: PortType, label?: string): PortSpec => ({
  ...port(type, label),
  optional: true,
});

export const many = (type: PortType, label?: string): PortSpec => ({
  ...opt(type, label),
  many: true,
});

export const CLIP_PORT = "clip";

const clipModifier = {
  inputs: { [CLIP_PORT]: port("clip", "Clip") },
  outputs: { [CLIP_PORT]: port("clip", "Clip") },
} as const;

export const nodeBase = <C extends NodeCategory, P extends NodePhase>(category: C, phase: P) => ({
  category,
  phase,
});

export const modifierBase = <C extends NodeCategory>(category: C) => ({
  ...nodeBase(category, "compile"),
  ...clipModifier,
});

export const fromElement = (meta: ElementMeta) => ({
  title: sentence(meta.title),
  phase: "compile" as const,
  element: meta.key,
});

export const clipInput = (inputs: CompileInputs, name = CLIP_PORT): ClipValue | undefined => {
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
