import { useEffect } from "react";
import { problemCheck, workflowProblems } from "@/shared/api/check";
import { firstIssueText } from "@/shared/api/issue-text";
import {
  apiFailure,
  apiOk,
  type ApiEdited,
  type ApiEditorReply,
  type ApiEditorRequest,
  type ApiProjectState,
  type ApiReply,
  type ApiRunState,
} from "@/shared/api/types";
import { errorMessage, NO_PROJECT } from "@/shared/errors";
import { connectionProblem } from "@/shared/graph";
import { edgeId, nodeIdFor } from "@/shared/ids";
import { getNodeDef } from "@/shared/nodes/registry";
import { workflowSettingsSchema, type Workflow, type WorkflowNode } from "@/shared/schema/workflow";
import { mapNode } from "@/shared/workflow-edits";
import { setEditorApi } from "../app/api-host";
import { useRunStore } from "../store/run";
import { newNodeParams, useWorkflowStore } from "../store/workflow";
import { beginRun } from "./run";
import { save, VERSION_UNCHOSEN } from "./save";

type Op = ApiEditorRequest["op"];
type RequestOf<O extends Op> = Extract<ApiEditorRequest, { op: O }>;
type Handler<O extends Op> = (
  request: RequestOf<O>,
  wf: Workflow,
) => ApiEditorReply<O> | Promise<ApiEditorReply<O>>;

function edited(id?: string): ApiReply<ApiEdited> {
  const { workflow, revision } = useWorkflowStore.getState();
  const problems = workflow ? workflowProblems(workflow) : [];
  return apiOk({ ...(id ? { id } : {}), revision, problems });
}

function runState(): ApiRunState {
  const { running, runId } = useRunStore.getState();
  return { running, runId };
}

function read(_request: unknown, wf: Workflow): ApiReply<ApiProjectState> {
  const { projectDir, revision, savedRevision } = useWorkflowStore.getState();
  return apiOk({
    dir: projectDir,
    workflow: wf,
    revision,
    saved: revision === savedRevision,
    ...runState(),
  });
}

const hasNode = (wf: Workflow, id: string) => wf.nodes.some((n) => n.id === id);

function nextPosition(nodes: WorkflowNode[]): { x: number; y: number } {
  if (!nodes.length) return { x: 0, y: 0 };
  return { x: Math.max(...nodes.map((n) => n.position.x)) + 300, y: 0 };
}

function applyNodePatch(node: WorkflowNode, patch: RequestOf<"updateNode">): WorkflowNode {
  const next: WorkflowNode = { ...node };
  const clearsLabel = patch.label === null || patch.label === "";
  if (clearsLabel) delete next.label;
  else if (patch.label) next.label = patch.label;
  if (patch.position) next.position = patch.position;
  if (patch.disabled === false) delete next.disabled;
  else if (patch.disabled) next.disabled = true;
  if (patch.params) next.params = { ...node.params, ...patch.params };
  return next;
}

const addNode: Handler<"addNode"> = (request, wf) => {
  if (!getNodeDef(request.type))
    return apiFailure(400, `There's no node type “${request.type}”. See GET /nodes.`);
  const id = request.id ?? nodeIdFor(request.type);
  if (hasNode(wf, id)) return apiFailure(400, `A node with the id “${id}” already exists.`);
  useWorkflowStore.getState().update((w) => ({
    ...w,
    nodes: [
      ...w.nodes,
      {
        id,
        type: request.type,
        position: request.position ?? nextPosition(w.nodes),
        params: newNodeParams(request.type, request.params),
        ...(request.label ? { label: request.label } : {}),
      },
    ],
  }));
  return edited(id);
};

const updateNode: Handler<"updateNode"> = (request, wf) => {
  if (!hasNode(wf, request.id)) return apiFailure(404, `There's no node “${request.id}”.`);
  useWorkflowStore
    .getState()
    .update((w) => mapNode(w, request.id, (n) => applyNodePatch(n, request)));
  return edited();
};

const removeNodes: Handler<"removeNodes"> = (request, wf) => {
  const missing = request.ids.find((id) => !hasNode(wf, id));
  if (missing) return apiFailure(404, `There's no node “${missing}”.`);
  useWorkflowStore.getState().removeNodes(request.ids);
  return edited();
};

const addEdge: Handler<"addEdge"> = (request, wf) => {
  const { source, sourceHandle, target, targetHandle } = request;
  const edge = { source, sourceHandle, target, targetHandle };
  const problem = connectionProblem(wf, edge);
  if (problem) return apiFailure(problem.notFound ? 404 : 400, problem.message);
  useWorkflowStore.getState().addEdge(edge);
  return edited(edgeId(edge));
};

const removeEdges: Handler<"removeEdges"> = (request, wf) => {
  const missing = request.ids.find((id) => !wf.edges.some((e) => e.id === id));
  if (missing) return apiFailure(404, `There's no connection “${missing}”.`);
  useWorkflowStore.getState().removeEdges(request.ids);
  return edited();
};

const setSettings: Handler<"setSettings"> = (request) => {
  const parsed = workflowSettingsSchema.partial().strict().safeParse(request.settings);
  if (!parsed.success) return apiFailure(400, firstIssueText(parsed.error));
  useWorkflowStore.getState().setSettings(parsed.data);
  return edited();
};

const run: Handler<"run"> = async (request) => {
  if (useRunStore.getState().running) return apiFailure(409, "A run is already going.");
  try {
    const runId = await beginRun({
      targetNodeIds: request.targetNodeIds,
      fromNodeId: request.fromNodeId,
    });
    return apiOk({ running: true, runId });
  } catch (e) {
    return apiFailure(409, errorMessage(e));
  }
};

const replace: Handler<"replace"> = (request) => {
  useWorkflowStore.getState().update(() => request.workflow);
  return edited();
};

const setVariables: Handler<"setVariables"> = (request) => {
  useWorkflowStore.getState().setVariables(request.variables);
  return edited();
};

const problems: Handler<"problems"> = (_request, wf) => apiOk(problemCheck(workflowProblems(wf)));

const undo: Handler<"undo"> = () => {
  useWorkflowStore.getState().undo();
  return edited();
};

const redo: Handler<"redo"> = () => {
  useWorkflowStore.getState().redo();
  return edited();
};

const saveProject: Handler<"save"> = async () =>
  (await save()) ? edited() : apiFailure(409, VERSION_UNCHOSEN);

const stop: Handler<"stop"> = async () => {
  await window.opsis.cancelRun();
  return apiOk(runState());
};

const HANDLERS: { [O in Op]: Handler<O> } = {
  open: read,
  read,
  replace,
  addNode,
  updateNode,
  removeNodes,
  addEdge,
  removeEdges,
  setSettings,
  setVariables,
  problems,
  undo,
  redo,
  save: saveProject,
  run,
  stop,
};

function dispatch<O extends Op>(request: RequestOf<O>, wf: Workflow) {
  const handler: Handler<O> = HANDLERS[request.op];
  return handler(request, wf);
}

async function handle(request: ApiEditorRequest): Promise<ApiEditorReply> {
  const wf = useWorkflowStore.getState().workflow;
  if (!wf) return apiFailure(409, NO_PROJECT);
  return dispatch(request, wf);
}

export function useApiBridge(): void {
  useEffect(() => setEditorApi(handle), []);
}
