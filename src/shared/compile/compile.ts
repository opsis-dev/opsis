import { describeParamIssue, isStructural, topoSort } from "../graph";
import { getNodeDef } from "../nodes/registry";
import { isOutputType } from "../nodes/output";
import type { TimelineSpec } from "../schema/timeline";
import type { CompileValue, NodeResult } from "../schema/values";
import type { Workflow } from "../schema/workflow";
import { interpolate } from "../vars";
import { clipToSpec, isClip, nodeSpans, type Span } from "./clip";
import { errorMessage } from "../errors";
import { projectFolderName } from "../project-folder";

export type CompileOptions = {
  results?: Record<string, NodeResult>;
};

export type CompileResult = {
  values: Record<string, Partial<Record<string, CompileValue>>>;
  errors: Record<string, string>;
  specs: Record<string, TimelineSpec>;
  order: string[];
};

export const PROJECT_NAME_VAR = "Project name";

export function workflowVars(wf: Pick<Workflow, "name" | "variables">): Record<string, string> {
  return { [PROJECT_NAME_VAR]: projectFolderName(wf.name), ...wf.variables };
}

export function compileWorkflow(wf: Workflow, opts: CompileOptions = {}): CompileResult {
  const { order } = topoSort(wf);
  const byId = new Map(wf.nodes.map((n) => [n.id, n]));
  const vars = workflowVars(wf);
  const values: CompileResult["values"] = {};
  const errors: CompileResult["errors"] = {};
  const specs: CompileResult["specs"] = {};

  for (const id of order) {
    const node = byId.get(id)!;
    if (isStructural(node)) continue;
    const def = getNodeDef(node.type);
    if (!def) {
      errors[id] = "This node type isn't available";
      continue;
    }

    const inputs: Partial<Record<string, CompileValue>> = {};
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

    if (node.disabled) {
      const out: Partial<Record<string, CompileValue>> = {};
      const firstClip = Object.values(inputs).find(isClip);
      for (const [portId, spec] of Object.entries(def.outputs)) {
        const match =
          Object.values(inputs).find((v) => v?.type === spec.type) ??
          (spec.type === "clip" ? firstClip : undefined);
        if (match) out[portId] = match;
      }
      values[id] = out;
      continue;
    }

    const parsed = def.params.safeParse(interpolate(node.params ?? {}, vars));
    if (!parsed.success) {
      const issue = parsed.error.issues[0];
      errors[id] = describeParamIssue(def.params.shape, issue);
      values[id] = {};
      continue;
    }

    try {
      const out =
        def.compile?.({
          nodeId: id,
          params: parsed.data,
          inputs,
          settings: wf.settings,
          result: opts.results?.[id],
          vars,
          workflow: wf,
        }) ??
        (opts.results?.[id]?.outputs as Partial<Record<string, CompileValue>> | undefined) ??
        {};
      values[id] = out;
      if (isOutputType(node.type)) {
        const tl = out.timeline;
        if (tl?.type === "timeline") specs[id] = tl.spec;
      }
    } catch (error) {
      errors[id] = errorMessage(error);
      values[id] = {};
    }
  }

  return { values, errors, specs, order };
}

export type PreviewTarget = {
  spec: TimelineSpec;
  spans: Record<string, Span>;
  sourceNodeId: string | null;
};

export function previewSpec(
  wf: Workflow,
  result: CompileResult,
  selectedNodeId?: string | null,
): PreviewTarget | null {
  const pick = (id: string): TimelineSpec | null => {
    const out = result.values[id];
    if (!out) return null;
    if (result.specs[id]) return result.specs[id];
    const clip = Object.values(out).find(isClip);
    return clip ? clipToSpec(clip, wf.settings) : null;
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
