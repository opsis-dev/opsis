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
import type { FontFamilyDef } from "@/shared/fonts/types";
import {
  addProjectFont,
  connect,
  groupNodes,
  pasteNodes,
  removeNodes,
  renameWorkflowVariable,
  ungroup,
} from "@/shared/workflow-edits";
import { platform } from "@/renderer/lib/platform";
import { cardSize } from "../editor/card-size";
import { useUiStore } from "./ui";

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
  mirrorProject: (dir: string, workflow: Workflow) => void;
  mirrorSaved: (workflow: Workflow) => void;
  beginGesture: () => void;
  endGesture: () => void;
};

type NodeParams = WorkflowNode["params"];

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

      mirrorProject: (dir, workflow) =>
        set({ projectDir: dir, workflow, revision: 0, savedRevision: 0 }),

      mirrorSaved: (workflow) => {
        if (workflow === get().workflow) return;
        set((s) => ({ workflow, revision: s.revision + 1, savedRevision: s.revision + 1 }));
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
          params: newNodeParams(type, params),
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

      removeNodes: (ids) => get().update((wf) => removeNodes(wf, ids)),

      addEdge: (edge) => get().update((wf) => connect(wf, edge)),

      removeEdges: (ids) => {
        const gone = new Set(ids);
        get().update((wf) => ({ ...wf, edges: wf.edges.filter((e) => !gone.has(e.id)) }));
      },

      setSettings: (patch) =>
        get().update((wf) => ({ ...wf, settings: { ...wf.settings, ...patch } })),
      addProjectFont: (def) => get().update((wf) => addProjectFont(wf, def)),
      setVariables: (variables) => get().update((wf) => ({ ...wf, variables })),
      renameVariable: (from, to) => get().update((wf) => renameWorkflowVariable(wf, from, to)),
      rename: (name) => get().update((wf) => ({ ...wf, name })),

      paste: (nodes, edges, offset) => {
        const current = get().workflow;
        if (!current) return [];
        const pasted = pasteNodes(current, nodes, edges, offset);
        get().update(() => pasted.workflow);
        return pasted.ids;
      },

      groupNodes: (ids, title = "Group") =>
        get().update((wf) =>
          groupNodes(wf, ids, { id: randomId("group"), title, sizeOf: cardSize }),
        ),

      ungroup: (groupId) => get().update((wf) => ungroup(wf, groupId)),
    }),
    {
      partialize: (s) => ({ workflow: s.workflow }),
      limit: 200,
      equality: (a, b) => a.workflow === b.workflow,
    },
  ),
);

function bumpRevisionOnUndo(s: WorkflowState, prev: WorkflowState): void {
  if (s.workflow && prev.workflow && s.workflow !== prev.workflow && s.revision === prev.revision)
    useWorkflowStore.setState({ revision: s.revision + 1 });
}

useWorkflowStore.subscribe(bumpRevisionOnUndo);

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

const hostDefaults = (type: string): NodeParams => getNodeDef(type)?.hostDefaults?.(platform) ?? {};

export const newNodeParams = (type: string, params?: NodeParams): NodeParams => ({
  ...defaultParams(type),
  ...hostDefaults(type),
  ...params,
});
