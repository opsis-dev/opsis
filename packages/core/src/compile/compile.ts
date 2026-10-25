import { clipToSpec, isClip } from "#core/compile/clip";
import { errorMessage } from "#core/errors";
import { topoSort } from "#core/graph";
import { CLIP_PORT, interpolatedParams, type NodeDef } from "#core/nodes/define";
import { parseParams } from "#core/nodes/params";
import { convertInput } from "#core/nodes/ports";
import { getNodeDef, isOutputType } from "#core/nodes/registry";
import type { TimelineSpec } from "#core/schema/timeline";
import type { CompileValue, NodeResult } from "#core/schema/values";
import type { Workflow, WorkflowEdge, WorkflowNode, WorkflowSettings } from "#core/schema/workflow";
import { nodeSpans, type FrameRange } from "#core/timeline/flatten";
import { workflowVars } from "#core/vars";

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

const mergeValues = (prev: CompileValue, next: CompileValue): CompileValue =>
  prev.type === "audio" && next.type === "audio"
    ? { ...prev, items: [...prev.items, ...next.items] }
    : next;

export function gatherInputs(
  wf: Workflow,
  def: NodeDef,
  id: string,
  valueOf: (edge: WorkflowEdge) => CompileValue | undefined,
): PortValues {
  const inputs: PortValues = {};
  for (const e of wf.edges) {
    if (e.target !== id) continue;
    const spec = def.inputs[e.targetHandle];
    const value = convertInput(valueOf(e), spec);
    if (!value) continue;
    const prev = inputs[e.targetHandle];
    inputs[e.targetHandle] = spec?.many && prev ? mergeValues(prev, value) : value;
  }
  return inputs;
}

function bypass(def: NodeDef, inputs: PortValues): PortValues {
  const out: PortValues = {};
  for (const [portId, spec] of Object.entries(def.outputs)) {
    const match = Object.keys(def.inputs)
      .map((input) => inputs[input])
      .find((v) => v?.type === spec.type);
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
  const parsed = parseParams(
    def.params,
    interpolatedParams(def, node.params, ctx.vars),
    node.params,
  );
  if (!parsed.ok) return { values: {}, error: parsed.problem };
  try {
    const values =
      def.compile?.({
        nodeId: node.id,
        type: node.type,
        label: node.label,
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
    const inputs = gatherInputs(wf, def, id, (e) => values[e.source]?.[e.sourceHandle]);
    if (node.disabled) {
      values[id] = bypass(def, inputs);
      continue;
    }
    const outcome = runCompile(wf, node, def, inputs, { vars, result: opts.results?.[id] });
    values[id] = outcome.values;
    const clip = inputs[CLIP_PORT];
    if (outcome.error !== undefined) errors[id] = outcome.error;
    else if (isOutputType(node.type) && isClip(clip)) specs[id] = clipToSpec(clip, wf.settings);
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
