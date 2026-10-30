import ELK, { type ElkNode, type ElkPort } from "elkjs/lib/elk.bundled.js";
import type { Workflow, WorkflowNode } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { inputsBeforeCompile } from "@/workflow/graph";
import { isGroupNode } from "@/workflow/nodes";
import { useWorkflowStore, workflowEdits } from "../store/workflow";
import { cardSize, measuredPorts, type MeasuredPort } from "./card-size";
import { LANE_GAP } from "./edge-lanes";

const elk = new ELK();
const LAYER_GAP = 80;
const NODE_GAP = 32;
const EDGE_NODE_GAP = 24;

type Side = MeasuredPort["side"];

export type CardShape = { width: number; height: number; ports: MeasuredPort[] | null };

function portId(nodeId: string, side: Side, port: string) {
  return `${nodeId}::${side}::${port}`;
}

function listedPorts(wf: Workflow, n: WorkflowNode): Array<{ handle: string; side: Side }> {
  const def = getNodeDef(n.type);
  if (!def) return [];
  return [
    ...Object.keys(inputsBeforeCompile(def, wf.edges, n.id)).map((handle) => ({
      handle,
      side: "in" as const,
    })),
    ...Object.keys(def.outputs).map((handle) => ({ handle, side: "out" as const })),
  ];
}

function edgeEnds(wf: Workflow, nodeId: string): Array<{ handle: string; side: Side }> {
  return wf.edges.flatMap((e) => [
    ...(e.source === nodeId ? [{ handle: e.sourceHandle, side: "out" as const }] : []),
    ...(e.target === nodeId ? [{ handle: e.targetHandle, side: "in" as const }] : []),
  ]);
}

function portsOf(wf: Workflow, n: WorkflowNode, shape: CardShape): ElkNode {
  const ports: Array<{ handle: string; side: Side; y?: number }> = [
    ...(shape.ports ?? listedPorts(wf, n)),
  ];
  const known = new Set(ports.map((p) => portId(n.id, p.side, p.handle)));
  for (const end of edgeEnds(wf, n.id)) {
    const id = portId(n.id, end.side, end.handle);
    if (known.has(id)) continue;
    known.add(id);
    ports.push(end);
  }
  const placed = ports.every((p) => p.y !== undefined);
  const outs = ports.filter((p) => p.side === "out");
  const ins = ports.filter((p) => p.side === "in");
  const clockwise = [...outs, ...ins.toReversed()];
  const elkPorts: ElkPort[] = clockwise.map((p, i) => ({
    id: portId(n.id, p.side, p.handle),
    width: 0,
    height: 0,
    ...(placed ? { x: p.side === "in" ? 0 : shape.width, y: p.y } : {}),
    layoutOptions: {
      "elk.port.side": p.side === "in" ? "WEST" : "EAST",
      "elk.port.index": String(i),
    },
  }));
  return {
    id: n.id,
    width: shape.width,
    height: shape.height,
    ports: elkPorts,
    layoutOptions: { "elk.portConstraints": placed ? "FIXED_POS" : "FIXED_ORDER" },
  };
}

export function layoutGraph(wf: Workflow, shapeOf: (n: WorkflowNode) => CardShape): ElkNode {
  const nodes = wf.nodes.filter((n) => !n.parentId && !isGroupNode(n));
  const ids = new Set(nodes.map((n) => n.id));
  return {
    id: "root",
    layoutOptions: {
      "elk.algorithm": "layered",
      "elk.direction": "RIGHT",
      "elk.edgeRouting": "ORTHOGONAL",
      "elk.layered.spacing.nodeNodeBetweenLayers": String(LAYER_GAP),
      "elk.layered.spacing.edgeEdgeBetweenLayers": String(LANE_GAP),
      "elk.layered.spacing.edgeNodeBetweenLayers": String(EDGE_NODE_GAP),
      "elk.spacing.nodeNode": String(NODE_GAP),
      "elk.layered.nodePlacement.strategy": "NETWORK_SIMPLEX",
    },
    children: nodes.map((n) => portsOf(wf, n, shapeOf(n))),
    edges: wf.edges
      .filter((e) => ids.has(e.source) && ids.has(e.target))
      .map((e) => ({
        id: e.id,
        sources: [portId(e.source, "out", e.sourceHandle)],
        targets: [portId(e.target, "in", e.targetHandle)],
      })),
  };
}

function shownShape(n: WorkflowNode): CardShape {
  return {
    ...cardSize(n),
    ports: measuredPorts(n.id),
  };
}

export async function autoLayout(
  wf: Workflow,
  shapeOf: (n: WorkflowNode) => CardShape = shownShape,
): Promise<Record<string, { x: number; y: number }>> {
  const result = await elk.layout(layoutGraph(wf, shapeOf));
  return Object.fromEntries(
    (result.children ?? []).map((c) => [c.id, { x: c.x ?? 0, y: c.y ?? 0 }]),
  );
}

export async function tidyLayout(): Promise<void> {
  const wf = useWorkflowStore.getState().workflow;
  if (!wf) return;
  workflowEdits.moveNodes(await autoLayout(wf));
}
