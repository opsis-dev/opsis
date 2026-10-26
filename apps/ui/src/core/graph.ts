import { findById } from "@opsis/video/common/lists";
import type { CatalogPort, Workflow } from "@/generated/bindings";
import type { EdgeEnds } from "@/generated/bindings";
import { canConnect, getNodeDef } from "@/store/catalog";

export function portOf(
  wf: Workflow,
  nodeId: string,
  side: "in" | "out",
  handle: string,
): CatalogPort | undefined {
  const node = findById(wf.nodes, nodeId);
  const def = node ? getNodeDef(node.type) : undefined;
  return def ? (side === "in" ? def.inputs : def.outputs)[handle] : undefined;
}

export function connects(wf: Workflow, c: EdgeEnds): boolean {
  if (c.source === c.target) return false;
  const from = portOf(wf, c.source, "out", c.sourceHandle);
  const to = portOf(wf, c.target, "in", c.targetHandle);
  return !!from && !!to && canConnect(from.type, to.type);
}
