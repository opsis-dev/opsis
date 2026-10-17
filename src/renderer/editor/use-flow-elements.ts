import {
  applyEdgeChanges,
  applyNodeChanges,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange,
} from "@xyflow/react";
import { useCallback, useMemo, useState } from "react";
import { portOf } from "@/shared/graph";
import type { Workflow } from "@/shared/schema/workflow";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { toFlowEdges, toFlowNodes, type FlowNodeData } from "./convert";
import { portTypeColor } from "./ports";

type FlowNode = Node<FlowNodeData>;
type LiveFields = Partial<
  Pick<FlowNode, "measured" | "position" | "dragging" | "resizing" | "width" | "height">
>;

const liveFields = (n: FlowNode): LiveFields =>
  n.dragging || n.resizing
    ? {
        measured: n.measured,
        position: n.position,
        dragging: n.dragging,
        resizing: n.resizing,
        width: n.width,
        height: n.height,
      }
    : { measured: n.measured };

const withLive = (base: FlowNode[], live: ReadonlyMap<string, LiveFields>): FlowNode[] =>
  base.map((n) => {
    const fields = live.get(n.id);
    return fields ? { ...n, ...fields } : n;
  });

function selectFromChanges(changes: NodeChange<FlowNode>[]): void {
  const picks = changes.flatMap((c) => (c.type === "select" ? [c] : []));
  if (!picks.length) return;
  const { selectedIds, setSelected } = useUiStore.getState();
  const next = new Set(selectedIds);
  for (const c of picks) {
    if (c.selected) next.add(c.id);
    else next.delete(c.id);
  }
  setSelected([...next]);
}

function keyboardMoves(
  changes: NodeChange<FlowNode>[],
  live: ReadonlyMap<string, LiveFields>,
): Record<string, { x: number; y: number }> {
  return Object.fromEntries(
    changes.flatMap((c) =>
      c.type === "position" && c.dragging === false && c.position && !live.get(c.id)?.dragging
        ? [[c.id, c.position]]
        : [],
    ),
  );
}

export function useFlowNodes(workflow: Workflow, selected: ReadonlySet<string>) {
  const base = useMemo(() => toFlowNodes(workflow, selected), [workflow, selected]);
  const [live, setLive] = useState<ReadonlyMap<string, LiveFields>>(() => new Map());
  const nodes = useMemo(() => withLive(base, live), [base, live]);
  const onNodesChange = useCallback(
    (changes: NodeChange<FlowNode>[]) => {
      const moved = keyboardMoves(changes, live);
      if (Object.keys(moved).length) useWorkflowStore.getState().moveNodes(moved);
      setLive((prev) => {
        const next = applyNodeChanges(changes, withLive(base, prev));
        return new Map(next.map((n) => [n.id, liveFields(n)]));
      });
      selectFromChanges(changes);
    },
    [base, live],
  );
  return { nodes, onNodesChange };
}

export function useFlowEdges(workflow: Workflow, active: ReadonlySet<string>) {
  const base = useMemo(
    () =>
      toFlowEdges(
        workflow,
        (e) => portTypeColor(portOf(workflow, e.source, "out", e.sourceHandle)?.type),
        active,
      ),
    [workflow, active],
  );
  const [picked, setPicked] = useState<ReadonlySet<string>>(() => new Set());
  const edges = useMemo(
    () => base.map((e): Edge => ({ ...e, selected: picked.has(e.id) })),
    [base, picked],
  );
  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) =>
      setPicked((prev) => {
        const current = base.map((e) => ({ ...e, selected: prev.has(e.id) }));
        const next = applyEdgeChanges(changes, current);
        return new Set(next.filter((e) => e.selected).map((e) => e.id));
      }),
    [base],
  );
  return { edges, onEdgesChange };
}
