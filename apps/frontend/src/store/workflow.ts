import { create } from "zustand";
import {
  commands,
  type DocumentState,
  type EdgeEnds,
  type Edited,
  type FontFamilyDef,
  type FontRole,
  type KeyframeEdit,
  type MarkerEdit,
  type MixEdit,
  type NodePatch,
  type Removal,
  type Position,
  type SettingsPatch,
  type Workflow,
  type WorkflowEdge,
  type WorkflowEdit,
  type WorkflowNode,
} from "@/generated/bindings";
import { reportError } from "@/lib/toast";
import { cardSize } from "../editor/card-size";
import { useUiStore } from "./ui";

type WorkflowState = {
  session: string | null;
  projectDir: string | null;
  workflow: Workflow | null;
  revision: number;
  savedRevision: number;
  canUndo: boolean;
  canRedo: boolean;
};

const NO_DOCUMENT: WorkflowState = {
  session: null,
  projectDir: null,
  workflow: null,
  revision: 0,
  savedRevision: 0,
  canUndo: false,
  canRedo: false,
};

export const useWorkflowStore = create<WorkflowState>()(() => NO_DOCUMENT);

export function showDocument(state: DocumentState): void {
  const current = useWorkflowStore.getState();
  if (current.session === state.session && state.revision < current.revision) return;
  useWorkflowStore.setState({
    session: state.session,
    projectDir: state.dir,
    workflow: state.workflow,
    revision: state.revision,
    savedRevision: state.savedRevision,
    canUndo: state.canUndo,
    canRedo: state.canRedo,
  });
}

export const editWorkflow = (edit: WorkflowEdit): Promise<Edited> => commands.editWorkflow(edit);

const CHANGE_FAILED = "Couldn't make that change";

function send(edit: WorkflowEdit): void {
  editWorkflow(edit).catch((e: unknown) => reportError(CHANGE_FAILED, e));
}

const created = (edit: WorkflowEdit): Promise<string[]> =>
  editWorkflow(edit).then(
    (edited) => edited.ids,
    (e: unknown) => {
      reportError(CHANGE_FAILED, e);
      return [];
    },
  );

const integerSize = (node: WorkflowNode) => {
  const { width, height } = cardSize(node);
  return { width: Math.round(width), height: Math.round(height) };
};

export const workflowEdits = {
  addNode: async (
    type: string,
    position: Position,
    params?: Record<string, unknown>,
  ): Promise<string | undefined> =>
    (await created({ op: "addNode", type, position, ...(params ? { params } : {}) }))[0],
  updateParams: (id: string, params: Record<string, unknown>) =>
    send({ op: "updateNode", id, patch: { params } }),
  setNodeMeta: (id: string, patch: NodePatch) => send({ op: "updateNode", id, patch }),
  moveNodes: (positions: Record<string, Position>) => send({ op: "moveNodes", positions }),
  removeNodes: (ids: string[]) => send({ op: "removeNodes", ids }),
  removeParts: ({ nodeIds, edgeIds }: Removal) => send({ op: "removeParts", nodeIds, edgeIds }),
  addEdge: (ends: EdgeEnds) => send({ op: "addEdge", ends }),
  removeEdges: (ids: string[]) => send({ op: "removeEdges", ids }),
  setSettings: (patch: SettingsPatch) => send({ op: "setSettings", patch }),
  addProjectFont: (font: FontFamilyDef) => send({ op: "addProjectFont", font }),
  setFontRole: (role: FontRole, family: string) => send({ op: "setFontRole", role, family }),
  setVariables: (variables: Record<string, string>) => send({ op: "setVariables", variables }),
  renameVariable: (from: string, to: string) => send({ op: "renameVariable", from, to }),
  rename: (name: string) => send({ op: "rename", name }),
  paste: (nodes: WorkflowNode[], edges: WorkflowEdge[], offset: Position): Promise<string[]> =>
    created({ op: "paste", nodes, edges, offset }),
  groupNodes: (ids: string[], title = "Group") => {
    const nodes = useWorkflowStore.getState().workflow?.nodes ?? [];
    const sizes = Object.fromEntries(
      nodes.filter((n) => ids.includes(n.id)).map((n) => [n.id, integerSize(n)]),
    );
    send({ op: "group", ids, title, sizes });
  },
  ungroup: (id: string) => send({ op: "ungroup", id }),
  mix: (edit: MixEdit) => send({ op: "mix", edit }),
  keyframes: (edit: KeyframeEdit) => send({ op: "keyframes", edit }),
  markers: async (edit: MarkerEdit): Promise<string | undefined> =>
    (await created({ op: "markers", edit }))[0],
  replace: (workflow: Workflow) => send({ op: "replace", workflow }),
  undo: () => void commands.undo(),
  redo: () => void commands.redo(),
  beginGesture: () => void commands.beginGesture(),
  endGesture: () => void commands.endGesture(),
};

useWorkflowStore.subscribe((s, prev) => {
  if (s.workflow && s.workflow.nodes !== prev.workflow?.nodes)
    useUiStore.getState().keepNodes(new Set(s.workflow.nodes.map((n) => n.id)));
});
