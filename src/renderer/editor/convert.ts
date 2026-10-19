import type { Edge, Node } from "@xyflow/react";
import { isGroupNode, isNoteNode } from "@/shared/graph";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";

type FlowNodeData = { node: WorkflowNode };

export type CanvasNode = Node<FlowNodeData>;

export type FlowNodeType = "opsis" | "group" | "note";

function toFlowNode(n: WorkflowNode, group: boolean, selected: ReadonlySet<string>): CanvasNode {
  const type: FlowNodeType = group ? "group" : isNoteNode(n) ? "note" : "opsis";
  return {
    id: n.id,
    type,
    position: n.position,
    data: { node: n },
    selected: selected.has(n.id),
    ...(n.parentId ? { parentId: n.parentId } : {}),
    ...(group && n.size
      ? { style: { width: n.size.width, height: n.size.height }, zIndex: -1 }
      : {}),
  };
}

export function toFlowNodes(wf: Workflow, selected: ReadonlySet<string>): CanvasNode[] {
  const groups: CanvasNode[] = [];
  const others: CanvasNode[] = [];
  for (const n of wf.nodes) {
    const group = isGroupNode(n);
    (group ? groups : others).push(toFlowNode(n, group, selected));
  }
  return [...groups, ...others];
}

export function toFlowEdges(
  wf: Workflow,
  colors: (edge: { source: string; sourceHandle: string }) => string,
  active: ReadonlySet<string>,
): Edge[] {
  return wf.edges.map((e) => ({
    id: e.id,
    source: e.source,
    sourceHandle: e.sourceHandle,
    target: e.target,
    targetHandle: e.targetHandle,
    animated: active.has(e.source) || active.has(e.target),
    style: { stroke: colors(e), strokeWidth: 2 },
  }));
}
