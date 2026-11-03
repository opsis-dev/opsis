import { findById } from "@opsis/video/common/lists";
import type {
  CatalogFamily,
  CatalogPort,
  EdgeEnds,
  EditorNode,
  PortType,
  Workflow,
  WorkflowEdge,
} from "@/generated/bindings";
import { canConnect, getNodeDef } from "@/store/catalog";

function inFamily(family: CatalogFamily, handle: string): boolean {
  return handle.startsWith(family.prefix) && /^[1-9]\d*$/.test(handle.slice(family.prefix.length));
}

function familyPort(def: EditorNode, handle: string): CatalogPort | undefined {
  const family = def.family;
  if (!family || !inFamily(family, handle)) return undefined;
  return { type: family.type, label: handle, optional: true, many: false };
}

export function inputsBeforeCompile(
  def: EditorNode,
  edges: readonly WorkflowEdge[],
  nodeId: string,
): Record<string, CatalogPort> {
  const family = def.family;
  if (!family) return def.inputs;
  const connected = edges
    .filter((e) => e.target === nodeId && inFamily(family, e.targetHandle))
    .map((e) => Number(e.targetHandle.slice(family.prefix.length)));
  if (!connected.length) return def.inputs;
  const inputs = { ...def.inputs };
  for (let i = 1; i <= Math.max(...connected) + 1; i++) {
    const handle = `${family.prefix}${i}`;
    const port = inputs[handle] ?? familyPort(def, handle);
    if (port) inputs[handle] = port;
  }
  return inputs;
}

export function portOf(
  wf: Workflow,
  nodeId: string,
  side: "in" | "out",
  handle: string,
): CatalogPort | undefined {
  const node = findById(wf.nodes, nodeId);
  const def = node ? getNodeDef(node.type) : undefined;
  if (!def) return undefined;
  if (side === "out") return def.outputs[handle];
  return def.inputs[handle] ?? familyPort(def, handle);
}

export type Converts = Partial<Record<string, Partial<Record<string, PortType[]>>>>;

export function connects(wf: Workflow, c: EdgeEnds, converts: Converts = {}): boolean {
  if (c.source === c.target) return false;
  const from = portOf(wf, c.source, "out", c.sourceHandle);
  const to = portOf(wf, c.target, "in", c.targetHandle);
  if (!from || !to || !canConnect(from.type, to.type)) return false;
  const now = converts[c.source]?.[c.sourceHandle];
  return !now || now.includes(to.type);
}
