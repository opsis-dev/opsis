import ELK from "elkjs/lib/elk.bundled.js";
import { getNodeDef } from "@/shared/nodes/registry";
import { CARD_WIDTH, estimateCardHeight } from "@/shared/nodes/size";
import type { Workflow } from "@/shared/schema/workflow";

const elk = new ELK();
const LAYER_GAP = 80;
const NODE_GAP = 32;

function measured(id: string): { width: number; height: number } | null {
  const el = document.querySelector<HTMLElement>(`.react-flow__node[data-id="${CSS.escape(id)}"]`);
  return el && el.offsetWidth && el.offsetHeight
    ? { width: el.offsetWidth, height: el.offsetHeight }
    : null;
}

export async function autoLayout(wf: Workflow): Promise<Record<string, { x: number; y: number }>> {
  const nodes = wf.nodes.filter((n) => !n.parentId && n.type !== "core.group");
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
      const size = measured(n.id) ?? { width: CARD_WIDTH, height: estimateCardHeight(def) };
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
