import {
  Background,
  BackgroundVariant,
  MiniMap,
  ReactFlow,
  type Connection,
  type IsValidConnection,
  type Node,
} from "@xyflow/react";
import { useCallback, useMemo, useRef } from "react";
import { connectionProblem } from "@/shared/graph";
import { ZoomSlider } from "@/renderer/components/zoom-slider";
import { isMac } from "../app/app-bar";
import { useAppSettings } from "../store/app-settings";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { SNAP_GRID } from "./canvas-grid";
import { CanvasToolbar } from "./canvas-toolbar";
import type { FlowNodeData } from "./convert";
import { FlowNode, GroupFlowNode, NoteFlowNode } from "./flow-node";
import { MAX_ZOOM, MIN_ZOOM } from "./pinch";
import { categoryColor } from "./ports";
import { useCanvasDrop } from "./use-canvas-drop";
import { useClipboardShortcuts } from "./use-clipboard-shortcuts";
import { useFlowEdges, useFlowNodes } from "./use-flow-elements";
import { useOpenWorkflow } from "./use-open-workflow";
import { usePinchZoom } from "./use-pinch-zoom";
import { useRevealNode } from "./use-reveal-node";

const nodeTypes = { demoflow: FlowNode, group: GroupFlowNode, note: NoteFlowNode };

const PRO_OPTIONS = { hideAttribution: true };

const minimapColor = (n: Node<FlowNodeData>) => categoryColor(n.data.node.type);

export function Canvas({ projectDir }: { projectDir: string }) {
  const workflow = useOpenWorkflow();
  const mode = useAppSettings((s) => s.settings?.editor.canvasMode ?? "mouse");
  const minimap = useUiStore((s) => s.minimap);
  const { addEdge, moveNodes, removeNodes, removeEdges } = useWorkflowStore.getState();
  const selectedIds = useUiStore((s) => s.selectedIds);
  const activeIds = useUiStore((s) => s.activeIds);
  const setSelected = useUiStore((s) => s.setSelected);
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

  const onConnect = useCallback(
    (c: Connection) => {
      if (!c.sourceHandle || !c.targetHandle) return;
      addEdge({
        source: c.source,
        sourceHandle: c.sourceHandle,
        target: c.target,
        targetHandle: c.targetHandle,
      });
    },
    [addEdge],
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
        onConnect={onConnect}
        isValidConnection={isValidConnection}
        onNodeDragStop={(_e, _node, dragged) =>
          moveNodes(Object.fromEntries(dragged.map((n) => [n.id, n.position])))
        }
        onNodesDelete={(deleted) => removeNodes(deleted.map((n) => n.id))}
        onEdgesDelete={(deleted) => removeEdges(deleted.map((e) => e.id))}
        onPaneClick={() => setSelected([])}
        deleteKeyCode={["Backspace", "Delete"]}
        multiSelectionKeyCode={["Meta", "Control", "Shift"]}
        snapToGrid
        snapGrid={[SNAP_GRID, SNAP_GRID]}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        minZoom={MIN_ZOOM}
        maxZoom={MAX_ZOOM}
        panOnScroll={mode === "touchpad"}
        zoomOnScroll={mode === "mouse"}
        zoomOnPinch
        proOptions={PRO_OPTIONS}
        colorMode="dark"
        defaultEdgeOptions={{ type: "smoothstep" }}
      >
        <Background variant={BackgroundVariant.Dots} gap={SNAP_GRID} size={1} />
        {minimap ? (
          <MiniMap<Node<FlowNodeData>> pannable zoomable nodeColor={minimapColor} />
        ) : null}
        <CanvasToolbar mode={mode} />
        <ZoomSlider position="bottom-left" />
      </ReactFlow>
    </div>
  );
}
