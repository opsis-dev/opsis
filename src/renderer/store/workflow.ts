import { temporal } from "zundo";
import { create } from "zustand";
import { defaultParams, getNodeDef } from "@/shared/nodes/registry";
import {
  normalizeWorkflow,
  type Marker,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
  type WorkflowSettings,
} from "@/shared/schema/workflow";
import { defaultMix, type Mix } from "@/shared/mix/types";
import { nodeIdFor, randomId } from "@/shared/ids";
import { renameVariable } from "@/shared/vars";
import { sameFamily } from "@/shared/fonts";
import type { FontFamilyDef } from "@/shared/fonts/types";
import { platform } from "@/renderer/lib/platform";
import { useUiStore } from "./ui";
import { isGroupNode } from "@/shared/graph";

type WorkflowState = {
  projectDir: string | null;
  workflow: Workflow | null;
  revision: number;
  savedRevision: number;
  fileVersion: string | null;
};

type Actions = {
  load: (dir: string, workflow: Workflow, fileVersion?: string | null) => void;
  setFileVersion: (version: string | null) => void;
  close: () => void;
  markSaved: (revision: number, fileVersion: string) => void;
  update: (fn: (wf: Workflow) => Workflow) => void;
  addNode: (type: string, position: { x: number; y: number }, params?: NodeParams) => string;
  updateParams: (id: string, patch: NodeParams) => void;
  setNodeMeta: (
    id: string,
    patch: Partial<Pick<WorkflowNode, "label" | "disabled" | "position" | "parentId" | "size">>,
  ) => void;
  moveNodes: (positions: Record<string, { x: number; y: number }>) => void;
  removeNodes: (ids: string[]) => void;
  addEdge: (edge: Omit<WorkflowEdge, "id">) => void;
  removeEdges: (ids: string[]) => void;
  setSettings: (patch: Partial<WorkflowSettings>) => void;
  addProjectFont: (def: FontFamilyDef) => void;
  setVariables: (vars: Record<string, string>) => void;
  renameVariable: (from: string, to: string) => void;
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

type NodeParams = WorkflowNode["params"];

const GROUP_PAD = 40;
const GROUP_TITLE_HEIGHT = 24;
const NODE_WIDTH = 260;
const NODE_HEIGHT = 140;

const WINDOW_STYLES: Partial<Record<string, string>> = { win32: "windows", linux: "linux" };

let gestureStart: Workflow | null = null;
let gestureDepth = 0;
let remote: ((message: RemoteEdit) => void) | null = null;

export type RemoteEdit =
  | { type: "replace"; workflow: Workflow }
  | { type: "gesture"; phase: "begin" | "end" };

export function mirrorEditsTo(send: ((message: RemoteEdit) => void) | null): void {
  remote = send;
}

const bump = (s: WorkflowState) => ({ revision: s.revision + 1 });

export const useWorkflowStore = create<WorkflowState & Actions>()(
  temporal(
    (set, get) => ({
      projectDir: null,
      workflow: null,
      revision: 0,
      savedRevision: 0,
      fileVersion: null,

      load: (dir, workflow, fileVersion = null) => {
        set({
          projectDir: dir,
          workflow: normalizeWorkflow(workflow),
          revision: 0,
          savedRevision: 0,
          fileVersion,
        });
        useWorkflowStore.temporal.getState().clear();
      },
      setFileVersion: (fileVersion) => set({ fileVersion }),
      close: () =>
        set({ projectDir: null, workflow: null, revision: 0, savedRevision: 0, fileVersion: null }),
      markSaved: (revision, fileVersion) => set({ savedRevision: revision, fileVersion }),

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
        commitAsOneStep(start, final);
      },

      setMix: (fn) => get().update((wf) => ({ ...wf, mix: fn(wf.mix ?? defaultMix()) })),

      setMarkers: (fn) => get().update((wf) => ({ ...wf, markers: fn(wf.markers) })),

      addNode: (type, position, params) => {
        const id = nodeIdFor(type);
        const node: WorkflowNode = {
          id,
          type,
          position,
          params: { ...defaultParams(type), ...hostDefaults(type), ...params },
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
      addProjectFont: (def) =>
        get().update((wf) => ({
          ...wf,
          settings: {
            ...wf.settings,
            fonts: {
              ...wf.settings.fonts,
              families: [
                ...wf.settings.fonts.families.filter((f) => !sameFamily(f.family, def.family)),
                def,
              ],
            },
          },
        })),
      setVariables: (variables) => get().update((wf) => ({ ...wf, variables })),
      renameVariable: (from, to) =>
        get().update((wf) => ({
          ...wf,
          variables: Object.fromEntries(
            Object.entries(wf.variables).map(([k, v]) => [k === from ? to : k, v]),
          ),
          nodes: to.trim()
            ? wf.nodes.map((n) => ({ ...n, params: renameVariable(n.params, from, to) }))
            : wf.nodes,
        })),
      rename: (name) => get().update((wf) => ({ ...wf, name })),

      paste: (nodes, edges, offset) => {
        const renamed = nodes.map((node) => ({ node, id: nodeIdFor(node.type) }));
        const map = new Map(renamed.map(({ node, id }) => [node.id, id]));
        const fresh = renamed.map(({ node: n, id }) => {
          const parentId = n.parentId ? map.get(n.parentId) : undefined;
          return {
            ...n,
            id,
            parentId,
            position: parentId
              ? n.position
              : { x: n.position.x + offset.x, y: n.position.y + offset.y },
          };
        });
        const freshEdges = edges.flatMap((e) => {
          const source = map.get(e.source);
          const target = map.get(e.target);
          if (!source || !target) return [];
          return [
            {
              ...e,
              source,
              target,
              id: `${source}.${e.sourceHandle}->${target}.${e.targetHandle}`,
            },
          ];
        });
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
          (n) => ids.includes(n.id) && !n.parentId && !isGroupNode(n),
        );
        if (members.length < 2) return;
        const xs = members.map((n) => n.position.x);
        const ys = members.map((n) => n.position.y);
        const x = Math.min(...xs) - GROUP_PAD;
        const y = Math.min(...ys) - GROUP_PAD - GROUP_TITLE_HEIGHT;
        const width = Math.max(...xs) - x + NODE_WIDTH + GROUP_PAD;
        const height = Math.max(...ys) - y + NODE_HEIGHT + GROUP_PAD;
        const groupId = randomId("group");
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

useWorkflowStore.subscribe((s, prev) => {
  if (s.workflow && prev.workflow && s.workflow !== prev.workflow && s.revision === prev.revision)
    useWorkflowStore.setState({ revision: s.revision + 1 });
});

useWorkflowStore.subscribe((s, prev) => {
  if (!s.workflow || s.workflow.nodes === prev.workflow?.nodes) return;
  const ids = new Set(s.workflow.nodes.map((n) => n.id));
  const { selectedIds, pinnedSource } = useUiStore.getState();
  const kept = selectedIds.filter((id) => ids.has(id));
  const pin = pinnedSource !== null && ids.has(pinnedSource) ? pinnedSource : null;
  if (kept.length !== selectedIds.length || pin !== pinnedSource)
    useUiStore.setState({ selectedIds: kept, pinnedSource: pin });
});

function commitAsOneStep(start: Workflow, final: Workflow): void {
  useWorkflowStore.setState({ workflow: start });
  useWorkflowStore.temporal.getState().resume();
  useWorkflowStore.setState({ workflow: final });
}

function hostDefaults(type: string): NodeParams {
  if (type !== "camera.deviceFrame") return {};
  return { windowStyle: WINDOW_STYLES[platform] ?? "macos" };
}
