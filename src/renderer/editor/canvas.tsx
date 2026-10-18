import {
  Background,
  BackgroundVariant,
  MiniMap,
  ReactFlow,
  type Connection,
  type Edge,
  type IsValidConnection,
  type ReactFlowProps,
} from "@xyflow/react";
import { useCallback, useMemo, useRef } from "react";
import { ZoomSlider } from "@/renderer/components/zoom-slider";
import { isMac } from "@/renderer/lib/platform";
import { connectionProblem } from "@/shared/graph";
import { DEFAULT_SETTINGS } from "@/shared/settings";
import { useAppSettings } from "../store/app-settings";
import { useUiStore } from "../store/ui";
import { useViewOptions } from "../store/view-options";
import { useWorkflowStore } from "../store/workflow";
import { SNAP_GRID } from "./canvas-grid";
import { CanvasToolbar } from "./canvas-toolbar";
import type { CanvasNode, FlowNodeType } from "./convert";
import { FlowNode, GroupFlowNode, NoteFlowNode } from "./flow-node";
import { MAX_ZOOM, MIN_ZOOM } from "./pinch";
import { categoryColor } from "./ports";
import { useCanvasDrop } from "./use-canvas-drop";
import { useClipboardShortcuts } from "./use-clipboard-shortcuts";
import { useFlowEdges, useFlowNodes } from "./use-flow-elements";
import { useOpenWorkflow } from "./use-open-workflow";
import { usePinchZoom } from "./use-pinch-zoom";
import { useRevealNode } from "./use-reveal-node";

const nodeTypes = {
  demoflow: FlowNode,
  group: GroupFlowNode,
  note: NoteFlowNode,
} satisfies Record<FlowNodeType, unknown>;

const FLOW_OPTIONS = {
  deleteKeyCode: ["Backspace", "Delete"],
  multiSelectionKeyCode: ["Meta", "Control", "Shift"],
  snapToGrid: true,
  snapGrid: [SNAP_GRID, SNAP_GRID],
  fitView: true,
  fitViewOptions: { padding: 0.2 },
  minZoom: MIN_ZOOM,
  maxZoom: MAX_ZOOM,
  zoomOnPinch: true,
  proOptions: { hideAttribution: true },
  colorMode: "dark",
  defaultEdgeOptions: { type: "smoothstep" },
} satisfies ReactFlowProps<CanvasNode, Edge>;

const minimapColor = (n: CanvasNode) => categoryColor(n.data.node.type);

function connect(c: Connection): void {
  if (!c.sourceHandle || !c.targetHandle) return;
  useWorkflowStore.getState().addEdge({
    source: c.source,
    sourceHandle: c.sourceHandle,
    target: c.target,
    targetHandle: c.targetHandle,
  });
}

const dropDragged = (_e: unknown, _node: CanvasNode, dragged: CanvasNode[]) =>
  useWorkflowStore.getState().moveNodes(Object.fromEntries(dragged.map((n) => [n.id, n.position])));

const deleteNodes = (deleted: CanvasNode[]) =>
  useWorkflowStore.getState().removeNodes(deleted.map((n) => n.id));

const deleteEdges = (deleted: Edge[]) =>
  useWorkflowStore.getState().removeEdges(deleted.map((e) => e.id));

const clearSelection = () => useUiStore.getState().setSelected([]);

export function Canvas({ projectDir }: { projectDir: string }) {
  const workflow = useOpenWorkflow();
  const mode = useAppSettings(
    (s) => s.settings?.editor.canvasMode ?? DEFAULT_SETTINGS.editor.canvasMode,
  );
  const minimap = useViewOptions((s) => s.minimap);
  const selectedIds = useUiStore((s) => s.selectedIds);
  const activeIds = useUiStore((s) => s.activeIds);
  const wrapper = useRef<HTMLDivElement>(null);
  usePinchZoom(wrapper, mode === "touchpad" && !isMac);
  useRevealNode(wrapper);
  useClipboardShortcuts(wrapper);
  const onDrop = useCanvasDrop(projectDir);

  const selected = useMemo(() => new Set(selectedIds), [selectedIds]);
  const active = useMemo(() => new Set(activeIds), [activeIds]);
  const { nodes, onNodesChange } = useFlowNodes(workflow, selected);
  const { edges, onEdgesChange } = useFlowEdges(workflow, active);

  const isValidConnection: IsValidConnection = useCallback(
    (c) =>
      !connectionProblem(workflow, {
        source: c.source,
        sourceHandle: c.sourceHandle ?? "",
        target: c.target,
        targetHandle: c.targetHandle ?? "",
      }),
    [workflow],
  );

  return (
    <div
      ref={wrapper}
      className="size-full"
      onDragOver={(e) => e.preventDefault()}
      onDrop={onDrop}
      tabIndex={-1}
    >
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={connect}
        isValidConnection={isValidConnection}
        onNodeDragStop={dropDragged}
        onNodesDelete={deleteNodes}
        onEdgesDelete={deleteEdges}
        onPaneClick={clearSelection}
        panOnScroll={mode === "touchpad"}
        zoomOnScroll={mode === "mouse"}
        {...FLOW_OPTIONS}
      >
        <Background variant={BackgroundVariant.Dots} gap={SNAP_GRID} size={1} />
        {minimap ? <MiniMap<CanvasNode> pannable zoomable nodeColor={minimapColor} /> : null}
        <CanvasToolbar mode={mode} />
        <ZoomSlider position="bottom-left" />
      </ReactFlow>
    </div>
  );
}
