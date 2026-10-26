import { downstreamOf } from "@/core/graph";
import type { Workflow, WorkflowNode } from "@/generated/bindings";

const incoming = (wf: Workflow, id: string): string =>
  JSON.stringify(
    wf.edges
      .filter((e) => e.target === id)
      .map((e) => [e.source, e.sourceHandle, e.targetHandle])
      .toSorted(),
  );

const sameNode = (a: WorkflowNode, b: WorkflowNode): boolean =>
  a.type === b.type &&
  !!a.disabled === !!b.disabled &&
  JSON.stringify(a.params) === JSON.stringify(b.params);

export function editedNodes(before: Workflow, after: Workflow): string[] {
  const old = new Map(before.nodes.map((n) => [n.id, n]));
  const changed = after.nodes
    .filter((n) => {
      const was = old.get(n.id);
      return !was || !sameNode(was, n) || incoming(before, n.id) !== incoming(after, n.id);
    })
    .map((n) => n.id);
  return [...new Set(changed.flatMap((id) => [id, ...downstreamOf(after, id)]))];
}
