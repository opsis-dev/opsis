import { upstreamOf } from "@/shared/graph";
import { getNodeDef } from "@/shared/nodes/registry";
import type { Workflow } from "@/shared/schema/workflow";

export function exportedClipNodes(wf: Workflow): string[] {
  const out = new Set<string>();
  for (const node of wf.nodes) {
    if (getNodeDef(node.type)?.phase !== "render") continue;
    for (const id of upstreamOf(wf, node.id)) {
      const def = wf.nodes.find((n) => n.id === id);
      if (def && "clip" in (getNodeDef(def.type)?.outputs ?? {})) out.add(id);
    }
  }
  return [...out];
}
