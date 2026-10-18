import { errorMessage } from "../errors";
import { topoSort } from "../graph";
import { interpolatedParams, type NodeDef } from "../nodes/define";
import { parseParams } from "../nodes/params";
import { isOutputType } from "../nodes/registry";
import { getNodeDef } from "../nodes/registry";
import { projectFolderName } from "../project-folder";
import type { TimelineSpec } from "../schema/timeline";
import type { CompileValue, NodeResult } from "../schema/values";
import type { Workflow, WorkflowNode, WorkflowSettings } from "../schema/workflow";
import { nodeSpans, type FrameRange } from "../timeline/flatten";
import { clipToSpec, isClip } from "./clip";

type CompileOptions = {
  results?: Partial<Record<string, NodeResult>>;
};

type PortValues = Partial<Record<string, CompileValue>>;

export type CompileResult = {
  values: Record<string, PortValues>;
  errors: Record<string, string>;
  specs: Record<string, TimelineSpec>;
  order: string[];
};

export const PROJECT_NAME_VAR = "Project name";

export function workflowVars(wf: Pick<Workflow, "name" | "variables">): Record<string, string> {
  return { [PROJECT_NAME_VAR]: projectFolderName(wf.name), ...wf.variables };
}

function gatherInputs(
  wf: Workflow,
  def: NodeDef,
  id: string,
  values: CompileResult["values"],
): PortValues {
  const inputs: PortValues = {};
  for (const e of wf.edges) {
    if (e.target !== id) continue;
    const v = values[e.source]?.[e.sourceHandle];
    if (!v) continue;
    const prev = inputs[e.targetHandle];
    inputs[e.targetHandle] =
      prev?.type === "audio" && v.type === "audio" && def.inputs[e.targetHandle]?.many
        ? { ...prev, items: [...prev.items, ...v.items] }
        : v;
  }
  return inputs;
}

function bypass(def: NodeDef, inputs: PortValues): PortValues {
  const out: PortValues = {};
  for (const [portId, spec] of Object.entries(def.outputs)) {
    const match = Object.values(inputs).find((v) => v?.type === spec.type);
    if (match) out[portId] = match;
  }
  return out;
}

type NodeOutcome = { values: PortValues; error?: string };

function runCompile(
  wf: Workflow,
  node: WorkflowNode,
  def: NodeDef,
  inputs: PortValues,
  ctx: { vars: Record<string, string>; result: NodeResult | undefined },
): NodeOutcome {
  const parsed = parseParams(def.params, interpolatedParams(def, node.params, ctx.vars));
  if (!parsed.ok) return { values: {}, error: parsed.problem };
  try {
    const values =
      def.compile?.({
        nodeId: node.id,
        params: parsed.data,
        inputs,
        settings: wf.settings,
        result: ctx.result,
        vars: ctx.vars,
        workflow: wf,
      }) ??
      ctx.result?.outputs ??
      {};
    return { values };
  } catch (error) {
    return { values: {}, error: errorMessage(error) };
  }
}

export function compileWorkflow(wf: Workflow, opts: CompileOptions = {}): CompileResult {
  const { sorted } = topoSort(wf);
  const vars = workflowVars(wf);
  const values: CompileResult["values"] = {};
  const errors: CompileResult["errors"] = {};
  const specs: CompileResult["specs"] = {};

  for (const node of sorted) {
    const { id } = node;
    const def = getNodeDef(node.type);
    if (!def) {
      errors[id] = "This node type isn't available";
      continue;
    }
    const inputs = gatherInputs(wf, def, id, values);
    if (node.disabled) {
      values[id] = bypass(def, inputs);
      continue;
    }
    const outcome = runCompile(wf, node, def, inputs, { vars, result: opts.results?.[id] });
    values[id] = outcome.values;
    if (outcome.error !== undefined) errors[id] = outcome.error;
    const tl = outcome.values.timeline;
    if (isOutputType(node.type) && tl?.type === "timeline") specs[id] = tl.spec;
  }

  return { values, errors, specs, order: sorted.map((n) => n.id) };
}

export type PreviewTarget = {
  spec: TimelineSpec;
  spans: Record<string, FrameRange>;
  sourceNodeId: string;
};

export function previewSpec(
  settings: WorkflowSettings,
  result: CompileResult,
  selectedNodeId?: string | null,
): PreviewTarget | null {
  const pick = (id: string): TimelineSpec | null => {
    const out = result.values[id];
    if (!out) return null;
    if (result.specs[id]) return result.specs[id];
    const clip = Object.values(out).find(isClip);
    return clip ? clipToSpec(clip, settings) : null;
  };
  const candidates = [
    ...(selectedNodeId ? [selectedNodeId] : []),
    ...Object.keys(result.specs),
    ...result.order.toReversed(),
  ];
  for (const id of candidates) {
    const spec = pick(id);
    if (spec) return { spec, spans: nodeSpans(spec), sourceNodeId: id };
  }
  return null;
}
