import {
  applyEdgeChanges,
  applyNodeChanges,
  type BuiltInEdge,
  type Edge,
  type EdgeChange,
  type NodeChange,
} from "@xyflow/react";
import { useCallback, useMemo, useState } from "react";
import type { Workflow } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { portOf } from "@/workflow/graph";
import { CARD_WIDTH } from "@/workflow/nodes";
import { useUiStore } from "../store/ui";
import { workflowEdits } from "../store/workflow";
import { toFlowEdges, toFlowNodes, type CanvasNode } from "./convert";
import { stepPositions, type LaneBox } from "./edge-lanes";
import { portTypeColor } from "./ports";

type LiveFields = Partial<
  Pick<CanvasNode, "measured" | "position" | "dragging" | "resizing" | "width" | "height">
>;

function liveFields(n: CanvasNode): LiveFields {
  return n.dragging || n.resizing
    ? {
        measured: n.measured,
        position: n.position,
        dragging: n.dragging,
        resizing: n.resizing,
        width: n.width,
        height: n.height,
      }
    : { measured: n.measured };
}

function withLive(base: CanvasNode[], live: ReadonlyMap<string, LiveFields>): CanvasNode[] {
  return base.map((n) => {
    const fields = live.get(n.id);
    return fields ? { ...n, ...fields } : n;
  });
}

function selectFromChanges(changes: NodeChange<CanvasNode>[]): void {
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
  changes: NodeChange<CanvasNode>[],
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
    (changes: NodeChange<CanvasNode>[]) => {
      const moved = keyboardMoves(changes, live);
      if (Object.keys(moved).length) workflowEdits.moveNodes(moved);
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

function laneBoxes(nodes: readonly CanvasNode[]): Map<string, LaneBox> {
  const byId = new Map(nodes.map((n) => [n.id, n]));
  function origin(n: CanvasNode | undefined): { x: number; y: number } {
    if (!n) return { x: 0, y: 0 };
    const parent = n.parentId ? origin(byId.get(n.parentId)) : { x: 0, y: 0 };
    return { x: parent.x + n.position.x, y: parent.y + n.position.y };
  }
  return new Map(
    nodes.map((n) => [
      n.id,
      {
        ...origin(n),
        width: n.measured?.width ?? CARD_WIDTH,
        height: n.measured?.height ?? getNodeDef(n.data.node.type)?.cardHeight ?? 0,
      },
    ]),
  );
}

export function useFlowEdges(
  workflow: Workflow,
  active: ReadonlySet<string>,
  nodes: readonly CanvasNode[],
) {
  const steps = useMemo(() => stepPositions(laneBoxes(nodes), workflow.edges), [nodes, workflow]);
  const base = useMemo(
    () =>
      toFlowEdges(
        workflow,
        (e) => portTypeColor(portOf(workflow, e.source, "out", e.sourceHandle)?.type),
        active,
      ).map((e): Edge => {
        const stepPosition = steps.get(e.id);
        if (stepPosition === undefined) return e;
        const laned: BuiltInEdge = { ...e, type: "smoothstep", pathOptions: { stepPosition } };
        return laned;
      }),
    [workflow, active, steps],
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
