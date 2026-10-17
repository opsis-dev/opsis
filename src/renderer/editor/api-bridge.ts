import { useEffect } from "react";
import { workflowProblems } from "@/shared/api/check";
import type {
  ApiEdited,
  ApiEditorReply,
  ApiEditorRequest,
  ApiFailure,
  ApiProjectState,
  ApiReply,
  ApiRunState,
} from "@/shared/api/types";
import { errorMessage } from "@/shared/errors";
import { connectionProblem } from "@/shared/graph";
import { randomId } from "@/shared/ids";
import { defaultParams, getNodeDef } from "@/shared/nodes/registry";
import { workflowSettingsSchema, type Workflow, type WorkflowNode } from "@/shared/schema/workflow";
import { setEditorApi } from "../app/api-host";
import { useRunStore } from "../store/run";
import { useWorkflowStore } from "../store/workflow";
import { beginRun, cancelRun } from "./run";
import { save } from "./save";

type Op = ApiEditorRequest["op"];
type RequestOf<O extends Op> = Extract<ApiEditorRequest, { op: O }>;
type Handler<O extends Op> = (
  request: RequestOf<O>,
  wf: Workflow,
) => ApiEditorReply<O> | Promise<ApiEditorReply<O>>;

const fail = (status: number, error: string): ApiFailure => ({ ok: false, status, error });
const done = <R>(result: R): ApiReply<R> => ({ ok: true, result });

function edited(id?: string): ApiReply<ApiEdited> {
  const { workflow, revision } = useWorkflowStore.getState();
  const problems = workflow ? workflowProblems(workflow) : [];
  return done({ ...(id ? { id } : {}), revision, problems });
}

function runState(): ApiRunState {
  const { running, runId } = useRunStore.getState();
  return { running, runId };
}

function read(wf: Workflow): ApiReply<ApiProjectState> {
  const { projectDir, revision, savedRevision } = useWorkflowStore.getState();
  return done({
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
  const { label, ...rest } = node;
  const next: WorkflowNode = { ...rest };
  if (patch.label === undefined) {
    if (label !== undefined) next.label = label;
  } else if (patch.label) next.label = patch.label;
  if (patch.position) next.position = patch.position;
  if (patch.disabled !== undefined) next.disabled = patch.disabled || undefined;
  if (patch.params) next.params = { ...node.params, ...patch.params };
  return next;
}

const addNode: Handler<"addNode"> = (request, wf) => {
  if (!getNodeDef(request.type))
    return fail(400, `There's no node type “${request.type}”. See GET /nodes.`);
  const id = request.id ?? randomId(request.type.split(".")[0]);
  if (hasNode(wf, id)) return fail(400, `A node with the id “${id}” already exists.`);
  useWorkflowStore.getState().update((w) => ({
    ...w,
    nodes: [
      ...w.nodes,
      {
        id,
        type: request.type,
        position: request.position ?? nextPosition(w.nodes),
        params: { ...defaultParams(request.type), ...request.params },
        ...(request.label ? { label: request.label } : {}),
      },
    ],
  }));
  return edited(id);
};

const updateNode: Handler<"updateNode"> = (request, wf) => {
  if (!hasNode(wf, request.id)) return fail(404, `There's no node “${request.id}”.`);
  useWorkflowStore.getState().update((w) => ({
    ...w,
    nodes: w.nodes.map((n) => (n.id === request.id ? applyNodePatch(n, request) : n)),
  }));
  return edited();
};

const removeNodes: Handler<"removeNodes"> = (request, wf) => {
  const missing = request.ids.find((id) => !hasNode(wf, id));
  if (missing) return fail(404, `There's no node “${missing}”.`);
  useWorkflowStore.getState().removeNodes(request.ids);
  return edited();
};

const addEdge: Handler<"addEdge"> = (request, wf) => {
  const { source, sourceHandle, target, targetHandle } = request;
  const edge = { source, sourceHandle, target, targetHandle };
  const problem = connectionProblem(wf, edge);
  if (problem) return fail(problem.notFound ? 404 : 400, problem.message);
  useWorkflowStore.getState().addEdge(edge);
  return edited(`${source}.${sourceHandle}->${target}.${targetHandle}`);
};

const removeEdges: Handler<"removeEdges"> = (request, wf) => {
  const missing = request.ids.find((id) => !wf.edges.some((e) => e.id === id));
  if (missing) return fail(404, `There's no connection “${missing}”.`);
  useWorkflowStore.getState().removeEdges(request.ids);
  return edited();
};

const setSettings: Handler<"setSettings"> = (request) => {
  const parsed = workflowSettingsSchema.partial().strict().safeParse(request.settings);
  if (!parsed.success) {
    const issue = parsed.error.issues[0];
    const where = issue.path.length ? `${issue.path.join(".")}: ` : "";
    return fail(400, `${where}${issue.message}`);
  }
  useWorkflowStore.getState().setSettings(parsed.data);
  return edited();
};

const run: Handler<"run"> = async (request) => {
  if (useRunStore.getState().running) return fail(409, "A run is already going.");
  try {
    const runId = await beginRun({
      targetNodeIds: request.targetNodeIds,
      fromNodeId: request.fromNodeId,
    });
    return done({ running: true, runId });
  } catch (e) {
    return fail(409, errorMessage(e));
  }
};

const HANDLERS: { [O in Op]: Handler<O> } = {
  open: (_request, wf) => read(wf),
  read: (_request, wf) => read(wf),
  replace: (request) => {
    useWorkflowStore.getState().update(() => request.workflow);
    return edited();
  },
  addNode,
  updateNode,
  removeNodes,
  addEdge,
  removeEdges,
  setSettings,
  setVariables: (request) => {
    useWorkflowStore.getState().setVariables(request.variables);
    return edited();
  },
  problems: (_request, wf) => {
    const problems = workflowProblems(wf);
    return done({ ok: !problems.some((p) => p.level === "error"), problems });
  },
  undo: () => {
    useWorkflowStore.temporal.getState().undo();
    return edited();
  },
  redo: () => {
    useWorkflowStore.temporal.getState().redo();
    return edited();
  },
  save: async () =>
    (await save())
      ? edited()
      : fail(409, "The project file changed on disk. Choose a version in the editor first."),
  run,
  stop: async () => {
    await cancelRun();
    return done(runState());
  },
};

function dispatch<O extends Op>(request: RequestOf<O>, wf: Workflow) {
  const handler: Handler<O> = HANDLERS[request.op];
  return handler(request, wf);
}

async function handle(request: ApiEditorRequest): Promise<ApiEditorReply> {
  const wf = useWorkflowStore.getState().workflow;
  if (!wf) return fail(409, "No project is open.");
  return dispatch(request, wf);
}

export function useApiBridge(): void {
  useEffect(() => setEditorApi(handle), []);
}
