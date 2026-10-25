import type { FontRole } from "@opsis/core/fonts";
import type { FontFamilyDef } from "@opsis/core/fonts/types";
import { nodeIdFor, randomId } from "@opsis/core/ids";
import { defaultMix, type Mix } from "@opsis/core/mix/types";
import { defaultParams, getNodeDef } from "@opsis/core/nodes/registry";
import {
  normalizeWorkflow,
  type EdgeEnds,
  type Marker,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
  type WorkflowSettings,
} from "@opsis/core/schema/workflow";
import { temporal } from "zundo";
import { create } from "zustand";
import { createForwarder } from "@/lib/forwarder";
import { platform } from "@/lib/platform";
import * as edits from "@/workflow-edits";
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
  addEdge: (edge: EdgeEnds) => void;
  removeEdges: (ids: string[]) => void;
  setSettings: (patch: Partial<WorkflowSettings>) => void;
  addProjectFont: (def: FontFamilyDef) => void;
  setFontRole: (role: FontRole, family: string) => void;
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
  applyPanelEdit: (workflow: Workflow) => void;
  mirrorProject: (dir: string, workflow: Workflow) => void;
  mirrorSaved: (workflow: Workflow) => void;
  undo: () => void;
  redo: () => void;
  beginGesture: () => void;
  endGesture: () => void;
};

type NodeParams = WorkflowNode["params"];

const MAX_UNDO_STEPS = 200;

export type RemoteEdit =
  | { type: "replace"; workflow: Workflow }
  | { type: "gesture"; phase: "begin" | "end" };

const remoteEdits = createForwarder<RemoteEdit>();

export const mirrorEditsTo = remoteEdits.to;

const opened = (projectDir: string | null, workflow: Workflow | null) => ({
  projectDir,
  workflow,
  revision: 0,
  savedRevision: 0,
});

function edited(
  s: WorkflowState,
  workflow: Workflow,
  { saved = false } = {},
): Partial<WorkflowState> {
  const revision = s.revision + 1;
  return saved ? { workflow, revision, savedRevision: revision } : { workflow, revision };
}

const history = () => useWorkflowStore.temporal.getState();

function stepHistory(step: "undo" | "redo"): void {
  const before = useWorkflowStore.getState().workflow;
  history()[step]();
  const after = useWorkflowStore.getState().workflow;
  if (before && after && before !== after)
    useWorkflowStore.setState((s) => ({ revision: s.revision + 1 }));
}

function gesturesAsOneUndoStep() {
  let depth = 0;
  let start: Workflow | null = null;
  return {
    begin: () => {
      if (depth++ > 0) return;
      start = useWorkflowStore.getState().workflow;
      history().pause();
    },
    end: () => {
      if (depth === 0 || --depth > 0) return;
      const from = start;
      start = null;
      const to = useWorkflowStore.getState().workflow;
      const changed = from !== null && to !== null && from !== to;
      if (changed) useWorkflowStore.setState({ workflow: from });
      history().resume();
      if (changed) useWorkflowStore.setState({ workflow: to });
    },
    drop: () => {
      if (depth === 0) return;
      depth = 0;
      start = null;
      history().resume();
    },
  };
}

const gestures = gesturesAsOneUndoStep();

export const useWorkflowStore = create<WorkflowState & Actions>()(
  temporal(
    (set, get) => ({
      projectDir: null,
      workflow: null,
      revision: 0,
      savedRevision: 0,
      fileVersion: null,

      load: (dir, workflow, fileVersion = null) => {
        gestures.drop();
        set({ ...opened(dir, normalizeWorkflow(workflow)), fileVersion });
        history().clear();
      },
      setFileVersion: (fileVersion) => set({ fileVersion }),
      markSaved: (revision, fileVersion) => set({ savedRevision: revision, fileVersion }),

      update: (fn) => {
        const current = get().workflow;
        if (!current) return;
        const next = fn(current);
        if (next === current) return;
        set((s) => edited(s, next));
        remoteEdits.send({ type: "replace", workflow: next });
      },

      applyPanelEdit: (workflow) => {
        if (workflow === get().workflow) return;
        set((s) => edited(s, workflow));
      },

      mirrorProject: (dir, workflow) => set(opened(dir, workflow)),

      mirrorSaved: (workflow) => {
        if (workflow === get().workflow) return;
        set((s) => edited(s, workflow, { saved: true }));
      },

      undo: () => stepHistory("undo"),

      redo: () => stepHistory("redo"),

      beginGesture: () => {
        if (!remoteEdits.send({ type: "gesture", phase: "begin" })) gestures.begin();
      },

      endGesture: () => {
        if (!remoteEdits.send({ type: "gesture", phase: "end" })) gestures.end();
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

      updateParams: (id, patch) => get().update((wf) => edits.patchNodeParams(wf, id, patch)),

      setNodeMeta: (id, patch) =>
        get().update((wf) => edits.mapNode(wf, id, (n) => ({ ...n, ...patch }))),

      moveNodes: (positions) =>
        get().update((wf) => ({
          ...wf,
          nodes: wf.nodes.map((n) => (positions[n.id] ? { ...n, position: positions[n.id] } : n)),
        })),

      removeNodes: (ids) => get().update((wf) => edits.removeNodes(wf, ids)),

      addEdge: (edge) => get().update((wf) => edits.connect(wf, edge)),

      removeEdges: (ids) => {
        const gone = new Set(ids);
        get().update((wf) => ({ ...wf, edges: wf.edges.filter((e) => !gone.has(e.id)) }));
      },

      setSettings: (patch) =>
        get().update((wf) => ({ ...wf, settings: { ...wf.settings, ...patch } })),
      addProjectFont: (def) => get().update((wf) => edits.addProjectFont(wf, def)),
      setFontRole: (role, family) => get().update((wf) => edits.setFontRole(wf, role, family)),
      setVariables: (variables) => get().update((wf) => ({ ...wf, variables })),
      renameVariable: (from, to) =>
        get().update((wf) => edits.renameWorkflowVariable(wf, from, to)),
      rename: (name) => get().update((wf) => ({ ...wf, name })),

      paste: (nodes, edges, offset) => {
        const current = get().workflow;
        if (!current) return [];
        const pasted = edits.pasteNodes(current, nodes, edges, offset);
        get().update(() => pasted.workflow);
        return pasted.ids;
      },

      groupNodes: (ids, title = "Group") =>
        get().update((wf) =>
          edits.groupNodes(wf, ids, { id: randomId("group"), title, sizeOf: cardSize }),
        ),

      ungroup: (groupId) => get().update((wf) => edits.ungroup(wf, groupId)),
    }),
    {
      partialize: (s) => ({ workflow: s.workflow }),
      limit: MAX_UNDO_STEPS,
      equality: (a, b) => a.workflow === b.workflow,
    },
  ),
);

useWorkflowStore.subscribe((s, prev) => {
  if (s.workflow && s.workflow.nodes !== prev.workflow?.nodes)
    useUiStore.getState().keepNodes(new Set(s.workflow.nodes.map((n) => n.id)));
});

const hostDefaults = (type: string): NodeParams => getNodeDef(type)?.hostDefaults?.(platform) ?? {};

export const newNodeParams = (type: string, params?: NodeParams): NodeParams => ({
  ...defaultParams(type),
  ...hostDefaults(type),
  ...params,
});
