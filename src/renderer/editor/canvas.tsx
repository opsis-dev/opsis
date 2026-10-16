import {
  applyEdgeChanges,
  applyNodeChanges,
  Background,
  BackgroundVariant,
  MiniMap,
  Panel,
  ReactFlow,
  useReactFlow,
  type Connection,
  type Edge,
  type EdgeChange,
  type IsValidConnection,
  type Node,
  type NodeChange,
} from "@xyflow/react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import { canConnect } from "@/shared/nodes/ports";
import { getNodeDef } from "@/shared/nodes/registry";
import type { WorkflowEdge, WorkflowNode } from "@/shared/schema/workflow";
import { ZoomSlider } from "@/renderer/components/zoom-slider";
import { Map as MapIcon, Mouse, Touchpad } from "lucide-react";
import type { CanvasMode } from "@/shared/settings";
import { Toggle } from "@/renderer/components/ui/toggle";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { patchAppSettings, useAppSettings } from "../store/app-settings";
import { MAX_ZOOM, MIN_ZOOM, pinchViewport } from "./pinch";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { toFlowEdges, toFlowNodes, type FlowNodeData } from "./convert";
import { FlowNode, GroupFlowNode, NoteFlowNode } from "./flow-node";
import { CATEGORY_COLORS, PORT_COLORS } from "./ports";

export const NODE_DRAG_TYPE = "application/demoflow-node";

const nodeTypes = { demoflow: FlowNode, group: GroupFlowNode, note: NoteFlowNode };

const CLIPBOARD_TAG = "demoflow/v1";

const PRO_OPTIONS = { hideAttribution: true };

function usePinchZoom(
  wrapper: React.RefObject<HTMLDivElement | null>,
  flow: ReturnType<typeof useReactFlow>,
  enabled: boolean,
): void {
  useEffect(() => {
    const el = wrapper.current;
    if (!el || !enabled) return;
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      e.stopPropagation();
      const box = el.getBoundingClientRect();
      const current = flow.getViewport();
      const next = pinchViewport(current, e.deltaY, e.clientX - box.left, e.clientY - box.top);
      if (next.zoom !== current.zoom) void flow.setViewport(next);
    };
    el.addEventListener("wheel", onWheel, { capture: true, passive: false });
    return () => el.removeEventListener("wheel", onWheel, { capture: true });
  }, [wrapper, flow, enabled]);
}

function CanvasToolbar({ mode }: { mode: CanvasMode }) {
  const minimap = useUiStore((s) => s.minimap);
  return (
    <Panel
      position="top-right"
      className="bg-primary-foreground flex items-center gap-1 rounded-md p-1"
    >
      <ToggleGroup
        size="sm"
        value={[mode]}
        onValueChange={(v) =>
          v[0] && void patchAppSettings({ editor: { canvasMode: v[0] as CanvasMode } })
        }
      >
        <Tooltip>
          <TooltipTrigger render={<ToggleGroupItem value="mouse" aria-label="Mouse" />}>
            <Mouse />
          </TooltipTrigger>
          <TooltipContent>
            Mouse: the wheel zooms, drag the background to move around
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger render={<ToggleGroupItem value="touchpad" aria-label="Touchpad" />}>
            <Touchpad />
          </TooltipTrigger>
          <TooltipContent>Touchpad: two fingers move around, pinch to zoom</TooltipContent>
        </Tooltip>
      </ToggleGroup>
      <Tooltip>
        <TooltipTrigger
          render={
            <Toggle
              size="sm"
              aria-label="Minimap"
              pressed={minimap}
              onPressedChange={(v) => useUiStore.getState().setMinimap(v)}
            />
          }
        >
          <MapIcon />
        </TooltipTrigger>
        <TooltipContent>{minimap ? "Hide the minimap" : "Show the minimap"}</TooltipContent>
      </Tooltip>
    </Panel>
  );
}

export function Canvas() {
  const workflow = useWorkflowStore((s) => s.workflow)!;
  const mode = useAppSettings((s) => s.settings?.editor.canvasMode ?? "mouse");
  const minimap = useUiStore((s) => s.minimap);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const { addNode, addEdge, moveNodes, removeNodes, removeEdges, paste } =
    useWorkflowStore.getState();
  const selectedIds = useUiStore((s) => s.selectedIds);
  const activeIds = useUiStore((s) => s.activeIds);
  const setSelected = useUiStore((s) => s.setSelected);
  const flow = useReactFlow();
  const wrapper = useRef<HTMLDivElement>(null);
  usePinchZoom(wrapper, flow, mode === "touchpad" && window.demoflow?.platform !== "darwin");
  const reveal = useUiStore((s) => s.reveal);
  useEffect(() => {
    if (!reveal) return;
    const node = flow.getNode(reveal.id);
    const internal = flow.getInternalNode(reveal.id);
    const box = wrapper.current?.getBoundingClientRect();
    if (!node || !internal || !box) return;
    const { x, y, zoom } = flow.getViewport();
    const abs = internal.internals.positionAbsolute;
    const left = abs.x * zoom + x;
    const top = abs.y * zoom + y;
    const width = (internal.measured.width ?? 260) * zoom;
    const height = (internal.measured.height ?? 120) * zoom;
    if (left >= 0 && top >= 0 && left + width <= box.width && top + height <= box.height) return;
    const target = node.parentId ? [{ id: node.parentId }, { id: node.id }] : [{ id: node.id }];
    void flow.fitView({
      nodes: target,
      duration: 300,
      maxZoom: Math.max(flow.getZoom(), 0.8),
      padding: 0.4,
    });
  }, [reveal, flow]);

  const selected = useMemo(() => new Set(selectedIds), [selectedIds]);
  const active = useMemo(() => new Set(activeIds), [activeIds]);

  const [nodes, setNodes] = useState<Node<FlowNodeData>[]>(() => toFlowNodes(workflow, selected));
  const [edges, setEdges] = useState<Edge[]>([]);

  useEffect(() => {
    // oxlint-disable-next-line react/set-state-in-effect
    setNodes((prev) => {
      const measured = new Map(prev.map((n) => [n.id, n.measured]));
      return toFlowNodes(workflow, selected).map((n) => ({ ...n, measured: measured.get(n.id) }));
    });
  }, [workflow, selected]);

  const portColor = useCallback(
    (e: { source: string; sourceHandle: string }) => {
      const src = workflow.nodes.find((n) => n.id === e.source);
      const type = src ? getNodeDef(src.type)?.outputs[e.sourceHandle]?.type : undefined;
      return type ? PORT_COLORS[type] : "#71717a";
    },
    [workflow.nodes],
  );

  useEffect(() => {
    // oxlint-disable-next-line react/set-state-in-effect
    setEdges((prev) => {
      const sel = new Set(prev.filter((e) => e.selected).map((e) => e.id));
      return toFlowEdges(workflow, portColor, active).map((e) => ({
        ...e,
        selected: sel.has(e.id),
      }));
    });
  }, [workflow, portColor, active]);

  const onNodesChange = useCallback(
    (changes: NodeChange<Node<FlowNodeData>>[]) => {
      setNodes((ns) => applyNodeChanges(changes, ns));
      const sel = changes.filter((c) => c.type === "select");
      if (sel.length) {
        const next = new Set(useUiStore.getState().selectedIds);
        for (const c of sel) {
          if (c.type !== "select") continue;
          if (c.selected) next.add(c.id);
          else next.delete(c.id);
        }
        setSelected([...next]);
      }
    },
    [setSelected],
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((es) => applyEdgeChanges(changes, es)),
    [],
  );

  const isValidConnection: IsValidConnection = useCallback(
    (c) => {
      const s = workflow.nodes.find((n) => n.id === c.source);
      const t = workflow.nodes.find((n) => n.id === c.target);
      if (!s || !t || s.id === t.id) return false;
      const sp = getNodeDef(s.type)?.outputs[c.sourceHandle ?? ""];
      const tp = getNodeDef(t.type)?.inputs[c.targetHandle ?? ""];
      return !!sp && !!tp && canConnect(sp.type, tp.type);
    },
    [workflow.nodes],
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

  const addAt = useCallback(
    (type: string, clientX: number, clientY: number, params?: Record<string, unknown>) => {
      const pos = flow.screenToFlowPosition({ x: clientX, y: clientY });
      const id = addNode(
        type,
        { x: Math.round(pos.x / 15) * 15, y: Math.round(pos.y / 15) * 15 },
        params,
      );
      setSelected([id]);
      return id;
    },
    [flow, addNode, setSelected],
  );

  const onDrop = useCallback(
    async (event: React.DragEvent) => {
      event.preventDefault();
      const type = event.dataTransfer.getData(NODE_DRAG_TYPE);
      if (type) {
        addAt(type, event.clientX, event.clientY);
        return;
      }
      const files = [...event.dataTransfer.files];
      if (!files.length || !projectDir) return;
      const paths = files.map((f) => window.demoflow.pathForFile(f)).filter(Boolean);
      const assets = await window.demoflow.importAssets(projectDir, paths);
      assets.forEach((asset, i) =>
        addAt("media.import", event.clientX + i * 30, event.clientY + i * 30, { path: asset }),
      );
      if (assets.length)
        toast.success(
          assets.length === 1
            ? "Added 1 file to the project"
            : `Added ${assets.length} files to the project`,
        );
    },
    [addAt, projectDir],
  );

  useEffect(() => {
    const el = wrapper.current;
    if (!el) return;
    const onKey = async (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest("input, textarea, [contenteditable=true]")) return;
      const mod = e.metaKey || e.ctrlKey;
      if (mod && e.key === "c") {
        const ids = new Set(useUiStore.getState().selectedIds);
        const wf = useWorkflowStore.getState().workflow;
        if (!wf || !ids.size) return;
        const payload = {
          tag: CLIPBOARD_TAG,
          nodes: wf.nodes.filter((n) => ids.has(n.id)),
          edges: wf.edges.filter((x) => ids.has(x.source) && ids.has(x.target)),
        };
        await navigator.clipboard.writeText(JSON.stringify(payload));
      } else if (mod && e.key === "v") {
        try {
          const data = JSON.parse(await navigator.clipboard.readText()) as {
            tag?: string;
            nodes: WorkflowNode[];
            edges: WorkflowEdge[];
          };
          if (data.tag !== CLIPBOARD_TAG) return;
          setSelected(paste(data.nodes, data.edges, { x: 60, y: 60 }));
        } catch {}
      } else if (mod && e.key === "d") {
        e.preventDefault();
        const ids = new Set(useUiStore.getState().selectedIds);
        const wf = useWorkflowStore.getState().workflow;
        if (!wf || !ids.size) return;
        setSelected(
          paste(
            wf.nodes.filter((n) => ids.has(n.id)),
            wf.edges.filter((x) => ids.has(x.source) && ids.has(x.target)),
            { x: 60, y: 60 },
          ),
        );
      }
    };
    el.addEventListener("keydown", onKey);
    return () => el.removeEventListener("keydown", onKey);
  }, [paste, setSelected]);

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
        snapGrid={[15, 15]}
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
        <Background variant={BackgroundVariant.Dots} gap={15} size={1} />
        {minimap ? (
          <MiniMap
            pannable
            zoomable
            nodeColor={(n) =>
              CATEGORY_COLORS[
                getNodeDef((n.data as FlowNodeData).node.type)?.category ?? "Utility"
              ] ?? "#71717a"
            }
          />
        ) : null}
        <CanvasToolbar mode={mode} />
        <ZoomSlider position="bottom-left" />
      </ReactFlow>
    </div>
  );
}
