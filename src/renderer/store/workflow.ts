import { temporal } from "zundo";
import { create } from "zustand";
import { defaultParams, getNodeDef, normalizeWorkflow } from "@/shared/nodes/registry";
import type {
  Marker,
  Workflow,
  WorkflowEdge,
  WorkflowNode,
  WorkflowSettings,
} from "@/shared/schema/workflow";
import { defaultMix, type Mix } from "@/shared/mix/types";

export type WorkflowState = {
  projectDir: string | null;
  workflow: Workflow | null;
  revision: number;
  savedRevision: number;
};

type Actions = {
  load: (dir: string, workflow: Workflow) => void;
  close: () => void;
  markSaved: (revision: number) => void;
  update: (fn: (wf: Workflow) => Workflow) => void;
  addNode: (
    type: string,
    position: { x: number; y: number },
    params?: Record<string, unknown>,
  ) => string;
  updateParams: (id: string, patch: Record<string, unknown>) => void;
  setNodeMeta: (
    id: string,
    patch: Partial<Pick<WorkflowNode, "label" | "disabled" | "position" | "parentId" | "size">>,
  ) => void;
  moveNodes: (positions: Record<string, { x: number; y: number }>) => void;
  removeNodes: (ids: string[]) => void;
  addEdge: (edge: Omit<WorkflowEdge, "id">) => void;
  removeEdges: (ids: string[]) => void;
  setSettings: (patch: Partial<WorkflowSettings>) => void;
  setVariables: (vars: Record<string, string>) => void;
  rename: (name: string) => void;
  paste: (
    nodes: WorkflowNode[],
    edges: WorkflowEdge[],
    offset: { x: number; y: number },
  ) => string[];
  groupNodes: (ids: string[], title?: string) => void;
  ungroup: (groupId: string) => void;
  setMix: (fn: (mix: Mix) => Mix) => void;
  setMarkers: (fn: (markers: Marker[]) => Marker[]) => void;
  replace: (workflow: Workflow) => void;
  beginGesture: () => void;
  endGesture: () => void;
};

let gestureStart: Workflow | null = null;
let gestureDepth = 0;
let remote: ((message: RemoteEdit) => void) | null = null;

export type RemoteEdit =
  | { type: "replace"; workflow: Workflow }
  | { type: "gesture"; phase: "begin" | "end" };

export function mirrorEditsTo(send: ((message: RemoteEdit) => void) | null): void {
  remote = send;
}

export const isGestureActive = () => gestureDepth > 0;

export const newId = (prefix = "n") => `${prefix}_${Math.random().toString(36).slice(2, 9)}`;

const bump = (s: WorkflowState) => ({ revision: s.revision + 1 });

export const useWorkflowStore = create<WorkflowState & Actions>()(
  temporal(
    (set, get) => ({
      projectDir: null,
      workflow: null,
      revision: 0,
      savedRevision: 0,

      load: (dir, workflow) => {
        set({
          projectDir: dir,
          workflow: normalizeWorkflow(workflow),
          revision: 0,
          savedRevision: 0,
        });
        useWorkflowStore.temporal.getState().clear();
      },
      close: () => set({ projectDir: null, workflow: null, revision: 0, savedRevision: 0 }),
      markSaved: (revision) => set({ savedRevision: revision }),

      update: (fn) => {
        const current = get().workflow;
        if (!current) return;
        const next = fn(current);
        if (next === current) return;
        set((s) => ({ workflow: next, ...bump(s) }));
        remote?.({ type: "replace", workflow: next });
      },

      replace: (workflow) => {
        if (workflow === get().workflow) return;
        set((s) => ({ workflow, ...bump(s) }));
      },

      beginGesture: () => {
        if (remote) {
          remote({ type: "gesture", phase: "begin" });
          return;
        }
        if (gestureDepth++ > 0) return;
        gestureStart = get().workflow;
        useWorkflowStore.temporal.getState().pause();
      },

      endGesture: () => {
        if (remote) {
          remote({ type: "gesture", phase: "end" });
          return;
        }
        if (gestureDepth === 0 || --gestureDepth > 0) return;
        const start = gestureStart;
        gestureStart = null;
        const final = get().workflow;
        if (!start || !final || start === final) {
          useWorkflowStore.temporal.getState().resume();
          return;
        }
        set({ workflow: start });
        useWorkflowStore.temporal.getState().resume();
        set({ workflow: final });
      },

      setMix: (fn) => get().update((wf) => ({ ...wf, mix: fn(wf.mix ?? defaultMix()) })),

      setMarkers: (fn) => get().update((wf) => ({ ...wf, markers: fn(wf.markers ?? []) })),

      addNode: (type, position, params) => {
        const def = getNodeDef(type);
        const id = newId(type.split(".")[0]);
        const node: WorkflowNode = {
          id,
          type,
          position,
          params: { ...defaultParams(type), ...hostDefaults(type), ...params },
          ...(def ? {} : {}),
        };
        get().update((wf) => ({ ...wf, nodes: [...wf.nodes, node] }));
        return id;
      },

      updateParams: (id, patch) =>
        get().update((wf) => ({
          ...wf,
          nodes: wf.nodes.map((n) =>
            n.id === id ? { ...n, params: { ...n.params, ...patch } } : n,
          ),
        })),

      setNodeMeta: (id, patch) =>
        get().update((wf) => ({
          ...wf,
          nodes: wf.nodes.map((n) => (n.id === id ? { ...n, ...patch } : n)),
        })),

      moveNodes: (positions) =>
        get().update((wf) => ({
          ...wf,
          nodes: wf.nodes.map((n) => (positions[n.id] ? { ...n, position: positions[n.id] } : n)),
        })),

      removeNodes: (ids) => {
        const gone = new Set(ids);
        get().update((wf) => ({
          ...wf,
          ...(wf.mix
            ? {
                mix: {
                  ...wf.mix,
                  channels: Object.fromEntries(
                    Object.entries(wf.mix.channels).filter(([id]) => !gone.has(id)),
                  ),
                },
              }
            : {}),
          nodes: wf.nodes
            .filter((n) => !gone.has(n.id))
            .map((n) => {
              if (!n.parentId || !gone.has(n.parentId)) return n;
              const parent = wf.nodes.find((p) => p.id === n.parentId);
              const { parentId: _p, ...rest } = n;
              return {
                ...rest,
                position: {
                  x: n.position.x + (parent?.position.x ?? 0),
                  y: n.position.y + (parent?.position.y ?? 0),
                },
              };
            }),
          edges: wf.edges.filter((e) => !gone.has(e.source) && !gone.has(e.target)),
        }));
      },

      addEdge: (edge) =>
        get().update((wf) => {
          const target = wf.nodes.find((n) => n.id === edge.target);
          const many = !!(target && getNodeDef(target.type)?.inputs[edge.targetHandle]?.many);
          const edges = wf.edges.filter((e) =>
            many
              ? !(
                  e.target === edge.target &&
                  e.targetHandle === edge.targetHandle &&
                  e.source === edge.source &&
                  e.sourceHandle === edge.sourceHandle
                )
              : !(e.target === edge.target && e.targetHandle === edge.targetHandle),
          );
          return {
            ...wf,
            edges: [
              ...edges,
              {
                ...edge,
                id: `${edge.source}.${edge.sourceHandle}->${edge.target}.${edge.targetHandle}`,
              },
            ],
          };
        }),

      removeEdges: (ids) => {
        const gone = new Set(ids);
        get().update((wf) => ({ ...wf, edges: wf.edges.filter((e) => !gone.has(e.id)) }));
      },

      setSettings: (patch) =>
        get().update((wf) => ({ ...wf, settings: { ...wf.settings, ...patch } })),
      setVariables: (variables) => get().update((wf) => ({ ...wf, variables })),
      rename: (name) => get().update((wf) => ({ ...wf, name })),

      paste: (nodes, edges, offset) => {
        const map = new Map(nodes.map((n) => [n.id, newId(n.type.split(".")[0])]));
        const fresh = nodes.map((n) => ({
          ...n,
          id: map.get(n.id)!,
          parentId: n.parentId && map.has(n.parentId) ? map.get(n.parentId) : undefined,
          position:
            n.parentId && map.has(n.parentId)
              ? n.position
              : { x: n.position.x + offset.x, y: n.position.y + offset.y },
        }));
        const freshEdges = edges
          .filter((e) => map.has(e.source) && map.has(e.target))
          .map((e) => ({
            ...e,
            source: map.get(e.source)!,
            target: map.get(e.target)!,
            id: `${map.get(e.source)}.${e.sourceHandle}->${map.get(e.target)}.${e.targetHandle}`,
          }));
        get().update((wf) => ({
          ...wf,
          nodes: [...wf.nodes, ...fresh],
          edges: [...wf.edges, ...freshEdges],
        }));
        return fresh.map((n) => n.id);
      },

      groupNodes: (ids, title = "Group") => {
        const wf = get().workflow;
        if (!wf) return;
        const members = wf.nodes.filter(
          (n) => ids.includes(n.id) && !n.parentId && n.type !== "core.group",
        );
        if (members.length < 2) return;
        const pad = 40;
        const xs = members.map((n) => n.position.x);
        const ys = members.map((n) => n.position.y);
        const x = Math.min(...xs) - pad;
        const y = Math.min(...ys) - pad - 24;
        const width = Math.max(...xs) - x + 260 + pad;
        const height = Math.max(...ys) - y + 140 + pad;
        const groupId = newId("group");
        get().update((w) => ({
          ...w,
          nodes: [
            {
              id: groupId,
              type: "core.group",
              position: { x, y },
              params: { ...defaultParams("core.group"), title },
              size: { width, height },
            },
            ...w.nodes.map((n) =>
              members.some((m) => m.id === n.id)
                ? {
                    ...n,
                    parentId: groupId,
                    position: { x: n.position.x - x, y: n.position.y - y },
                  }
                : n,
            ),
          ],
        }));
      },

      ungroup: (groupId) => {
        const wf = get().workflow;
        const group = wf?.nodes.find((n) => n.id === groupId);
        if (!wf || !group) return;
        get().update((w) => ({
          ...w,
          nodes: w.nodes
            .filter((n) => n.id !== groupId)
            .map((n) => {
              if (n.parentId !== groupId) return n;
              const { parentId: _p, ...rest } = n;
              return {
                ...rest,
                position: {
                  x: n.position.x + group.position.x,
                  y: n.position.y + group.position.y,
                },
              };
            }),
        }));
      },
    }),
    {
      partialize: (s) => ({ workflow: s.workflow }),
      limit: 200,
      equality: (a, b) => a.workflow === b.workflow,
    },
  ),
);

export const useTemporal = () => useWorkflowStore.temporal.getState();

useWorkflowStore.subscribe((s, prev) => {
  if (s.workflow && prev.workflow && s.workflow !== prev.workflow && s.revision === prev.revision)
    useWorkflowStore.setState({ revision: s.revision + 1 });
});

function hostDefaults(type: string): Record<string, unknown> {
  if (type !== "camera.deviceFrame") return {};
  const platform = window.demoflow?.platform;
  return {
    windowStyle: platform === "win32" ? "windows" : platform === "linux" ? "linux" : "macos",
  };
}
