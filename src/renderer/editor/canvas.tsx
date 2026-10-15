import {
  applyEdgeChanges,
  applyNodeChanges,
  Background,
  BackgroundVariant,
  MiniMap,
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
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { toFlowEdges, toFlowNodes, type FlowNodeData } from "./convert";
import { FlowNode, GroupFlowNode, NoteFlowNode } from "./flow-node";
import { CATEGORY_COLORS, PORT_COLORS } from "./ports";

export const NODE_DRAG_TYPE = "application/demoflow-node";

const nodeTypes = { demoflow: FlowNode, group: GroupFlowNode, note: NoteFlowNode };

const CLIPBOARD_TAG = "demoflow/v1";

export function Canvas() {
  const workflow = useWorkflowStore((s) => s.workflow)!;
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const { addNode, addEdge, moveNodes, removeNodes, removeEdges, paste } =
    useWorkflowStore.getState();
  const selectedIds = useUiStore((s) => s.selectedIds);
  const activeIds = useUiStore((s) => s.activeIds);
  const setSelected = useUiStore((s) => s.setSelected);
  const flow = useReactFlow();
  const wrapper = useRef<HTMLDivElement>(null);

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
        minZoom={0.1}
        maxZoom={2}
        colorMode="dark"
        defaultEdgeOptions={{ type: "smoothstep" }}
      >
        <Background variant={BackgroundVariant.Dots} gap={15} size={1} />
        <MiniMap
          pannable
          zoomable
          nodeColor={(n) =>
            CATEGORY_COLORS[
              getNodeDef((n.data as FlowNodeData).node.type)?.category ?? "Utility"
            ] ?? "#71717a"
          }
        />
        <ZoomSlider position="bottom-left" />
      </ReactFlow>
    </div>
  );
}
