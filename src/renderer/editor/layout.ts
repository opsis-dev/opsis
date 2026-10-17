import ELK from "elkjs/lib/elk.bundled.js";
import { getNodeDef } from "@/shared/nodes/registry";
import type { Workflow } from "@/shared/schema/workflow";
import { useWorkflowStore } from "../store/workflow";
import { cardSize } from "./card-size";
import { isGroupNode } from "@/shared/graph";

const elk = new ELK();
const LAYER_GAP = 80;
const NODE_GAP = 32;

async function autoLayout(wf: Workflow): Promise<Record<string, { x: number; y: number }>> {
  const nodes = wf.nodes.filter((n) => !n.parentId && !isGroupNode(n));
  const ids = new Set(nodes.map((n) => n.id));
  const graph = {
    id: "root",
    layoutOptions: {
      "elk.algorithm": "layered",
      "elk.direction": "RIGHT",
      "elk.layered.spacing.nodeNodeBetweenLayers": String(LAYER_GAP),
      "elk.spacing.nodeNode": String(NODE_GAP),
      "elk.layered.nodePlacement.strategy": "NETWORK_SIMPLEX",
      "elk.portConstraints": "FIXED_ORDER",
    },
    children: nodes.map((n) => {
      const def = getNodeDef(n.type);
      const inputs = Object.keys(def?.inputs ?? {});
      const outputs = Object.keys(def?.outputs ?? {});
      const size = cardSize(n);
      return {
        id: n.id,
        width: size.width,
        height: size.height,
        ports: [
          ...inputs.map((p, i) => ({
            id: `${n.id}::in::${p}`,
            layoutOptions: { "elk.port.side": "WEST", "elk.port.index": String(i) },
          })),
          ...outputs.map((p, i) => ({
            id: `${n.id}::out::${p}`,
            layoutOptions: { "elk.port.side": "EAST", "elk.port.index": String(i) },
          })),
        ],
      };
    }),
    edges: wf.edges
      .filter((e) => ids.has(e.source) && ids.has(e.target))
      .map((e) => ({
        id: e.id,
        sources: [`${e.source}::out::${e.sourceHandle}`],
        targets: [`${e.target}::in::${e.targetHandle}`],
      })),
  };
  const result = await elk.layout(graph);
  return Object.fromEntries(
    (result.children ?? []).map((c) => [c.id, { x: c.x ?? 0, y: c.y ?? 0 }]),
  );
}

export async function tidyLayout(): Promise<void> {
  const wf = useWorkflowStore.getState().workflow;
  if (!wf) return;
  useWorkflowStore.getState().moveNodes(await autoLayout(wf));
}
