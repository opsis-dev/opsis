import { findById } from "@opsis/video/common/lists";
import type { CatalogPort, Workflow } from "@/generated/bindings";
import type { EdgeEnds } from "@/generated/bindings";
import { canConnect, getNodeDef } from "@/store/catalog";

function reach(wf: Workflow, nodeId: string, direction: "up" | "down"): Set<string> {
  const links = new Map<string, string[]>();
  for (const e of wf.edges) {
    const [from, to] = direction === "up" ? [e.target, e.source] : [e.source, e.target];
    const list = links.get(from);
    if (list) list.push(to);
    else links.set(from, [to]);
  }
  const out = new Set<string>();
  const stack = [nodeId];
  for (let id = stack.pop(); id !== undefined; id = stack.pop())
    for (const linked of links.get(id) ?? [])
      if (!out.has(linked)) {
        out.add(linked);
        stack.push(linked);
      }
  return out;
}

export const downstreamOf = (wf: Workflow, nodeId: string): Set<string> =>
  reach(wf, nodeId, "down");

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
