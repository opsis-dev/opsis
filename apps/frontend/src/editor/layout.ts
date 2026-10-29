import ELK from "elkjs/lib/elk.bundled.js";
import type { Workflow } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { isGroupNode } from "@/workflow/nodes";
import { useWorkflowStore, workflowEdits } from "../store/workflow";
import { cardSize } from "./card-size";

const elk = new ELK();
const LAYER_GAP = 80;
const NODE_GAP = 32;

type Side = "in" | "out";

const portId = (nodeId: string, side: Side, port: string) => `${nodeId}::${side}::${port}`;

const portsFor = (nodeId: string, names: string[], side: Side) =>
  names.map((p, i) => ({
    id: portId(nodeId, side, p),
    layoutOptions: {
      "elk.port.side": side === "in" ? "WEST" : "EAST",
      "elk.port.index": String(i),
    },
  }));

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
      const size = cardSize(n);
      return {
        id: n.id,
        width: size.width,
        height: size.height,
        ports: [
          ...portsFor(n.id, Object.keys(def?.inputs ?? {}), "in"),
          ...portsFor(n.id, Object.keys(def?.outputs ?? {}), "out"),
        ],
      };
    }),
    edges: wf.edges
      .filter((e) => ids.has(e.source) && ids.has(e.target))
      .map((e) => ({
        id: e.id,
        sources: [portId(e.source, "out", e.sourceHandle)],
        targets: [portId(e.target, "in", e.targetHandle)],
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
  workflowEdits.moveNodes(await autoLayout(wf));
}
