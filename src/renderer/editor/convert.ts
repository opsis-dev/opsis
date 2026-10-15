import type { Edge, Node } from "@xyflow/react";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";

export type FlowNodeData = { node: WorkflowNode };

export function toFlowNodes(wf: Workflow, selected: Set<string>): Node<FlowNodeData>[] {
  const ordered = [...wf.nodes].sort(
    (a, b) => (a.type === "core.group" ? -1 : 0) - (b.type === "core.group" ? -1 : 0),
  );
  return ordered.map((n) => ({
    id: n.id,
    type: n.type === "core.group" ? "group" : n.type === "core.note" ? "note" : "demoflow",
    position: n.position,
    data: { node: n },
    selected: selected.has(n.id),
    ...(n.parentId ? { parentId: n.parentId, extent: undefined } : {}),
    ...(n.type === "core.group" && n.size
      ? { style: { width: n.size.width, height: n.size.height }, zIndex: -1 }
      : {}),
  }));
}

export function toFlowEdges(
  wf: Workflow,
  colors: (edge: { source: string; sourceHandle: string }) => string,
  active: Set<string>,
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
