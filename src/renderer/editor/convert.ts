import type { Edge, Node } from "@xyflow/react";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";
import { isGroupNode, isNoteNode } from "@/shared/graph";

export type FlowNodeData = { node: WorkflowNode };

export type CanvasNode = Node<FlowNodeData>;

export type FlowNodeType = "demoflow" | "group" | "note";

function flowNodeType(node: WorkflowNode): FlowNodeType {
  if (isGroupNode(node)) return "group";
  return isNoteNode(node) ? "note" : "demoflow";
}

export function toFlowNodes(wf: Workflow, selected: ReadonlySet<string>): CanvasNode[] {
  const ordered = [...wf.nodes.filter(isGroupNode), ...wf.nodes.filter((n) => !isGroupNode(n))];
  return ordered.map((n) => ({
    id: n.id,
    type: flowNodeType(n),
    position: n.position,
    data: { node: n },
    selected: selected.has(n.id),
    ...(n.parentId ? { parentId: n.parentId } : {}),
    ...(isGroupNode(n) && n.size
      ? { style: { width: n.size.width, height: n.size.height }, zIndex: -1 }
      : {}),
  }));
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
