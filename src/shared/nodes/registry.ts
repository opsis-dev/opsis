import { audioNodes } from "./audio";
import { browserNodes } from "./browser";
import { cameraNodes } from "./camera";
import { captureNodes } from "./capture";
import { cursorNodes } from "./cursor";
import type { Workflow, WorkflowNode } from "../schema/workflow";
import { renameVariable } from "../vars";
import type { NodeDef } from "./define";
import { elementNodes } from "./elements";
import { mediaNodes } from "./media";
import { outputNodes } from "./output";
import { textNodes } from "./text";
import { timelineNodes } from "./timeline";
import { transitionNodes } from "./transitions";
import { utilityNodes } from "./utility";

export const NODE_DEFS: NodeDef[] = [
  ...browserNodes,
  ...captureNodes,
  ...cameraNodes,
  ...cursorNodes,
  ...textNodes,
  ...elementNodes,
  ...transitionNodes,
  ...audioNodes,
  ...timelineNodes,
  ...mediaNodes,
  ...outputNodes,
  ...utilityNodes,
];

const byType = new Map(NODE_DEFS.map((d) => [d.type, d]));

if (byType.size !== NODE_DEFS.length) {
  const seen = new Set<string>();
  const dupes = NODE_DEFS.filter((d) => (seen.has(d.type) ? true : (seen.add(d.type), false))).map(
    (d) => d.type,
  );
  throw new Error(`Duplicate node types: ${dupes.join(", ")}`);
}

export const getNodeDef = (type: string): NodeDef | undefined => byType.get(type);

export function defaultParams(type: string): Record<string, unknown> {
  const def = getNodeDef(type);
  if (!def) return {};
  const out: Record<string, unknown> = {};
  for (const [key, field] of Object.entries(def.params.shape)) {
    const parsed = (
      field as { safeParse: (v: unknown) => { success: boolean; data?: unknown } }
    ).safeParse(undefined);
    if (parsed.success && parsed.data !== undefined) out[key] = parsed.data;
  }
  return out;
}

export function normalizeNode(node: WorkflowNode): WorkflowNode {
  const def = getNodeDef(node.type);
  const label = node.params?.label;
  if (!def || typeof label !== "string" || "label" in def.params.shape) return node;
  const { label: _moved, ...params } = node.params;
  const name = node.label?.trim() || label.trim();
  return { ...node, params, ...(name ? { label: name } : {}) };
}

const LEGACY_VARIABLES: Record<string, string> = {
  BASE_URL: "Site address",
  PRODUCT_NAME: "Product name",
  VERSION: "Version",
  BEFORE_URL: "Before address",
  AFTER_URL: "After address",
};

export function normalizeWorkflow(wf: Workflow): Workflow {
  let nodes = wf.nodes.map(normalizeNode);
  const variables: Record<string, string> = {};
  for (const [name, value] of Object.entries(wf.variables)) {
    const next = LEGACY_VARIABLES[name];
    if (next && !(next in wf.variables)) {
      variables[next] = value;
      nodes = nodes.map((n) => ({ ...n, params: renameVariable(n.params, name, next) }));
    } else variables[name] = value;
  }
  nodes = nodes.map((n) => ({
    ...n,
    params: renameVariable(n.params, "PROJECT_NAME", "Project name"),
  }));
  return { ...wf, variables, nodes };
}
