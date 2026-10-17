import type { Edge, Node } from "@xyflow/react";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";

export type FlowNodeData = { node: WorkflowNode };

const FLOW_TYPES: Partial<Record<string, string>> = { "core.group": "group", "core.note": "note" };

const isGroup = (n: WorkflowNode) => n.type === "core.group";

export function toFlowNodes(wf: Workflow, selected: ReadonlySet<string>): Node<FlowNodeData>[] {
  const ordered = [...wf.nodes.filter(isGroup), ...wf.nodes.filter((n) => !isGroup(n))];
  return ordered.map((n) => ({
    id: n.id,
    type: FLOW_TYPES[n.type] ?? "demoflow",
    position: n.position,
    data: { node: n },
    selected: selected.has(n.id),
    ...(n.parentId ? { parentId: n.parentId } : {}),
    ...(isGroup(n) && n.size
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
