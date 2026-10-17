import type { Workflow, WorkflowSettings } from "../schema/workflow";

export type ApiProblem = {
  level: "error" | "warning";
  message: string;
  nodeId?: string;
  edgeId?: string;
  path?: string;
};

export type ApiEditorRequest =
  | { op: "open"; dir: string }
  | { op: "read" }
  | { op: "replace"; workflow: Workflow }
  | {
      op: "addNode";
      type: string;
      id?: string;
      label?: string;
      position?: { x: number; y: number };
      params?: Record<string, unknown>;
    }
  | {
      op: "updateNode";
      id: string;
      label?: string | null;
      position?: { x: number; y: number };
      disabled?: boolean;
      params?: Record<string, unknown>;
    }
  | { op: "removeNodes"; ids: string[] }
  | {
      op: "addEdge";
      source: string;
      sourceHandle: string;
      target: string;
      targetHandle: string;
    }
  | { op: "removeEdges"; ids: string[] }
  | { op: "setSettings"; settings: Partial<WorkflowSettings> }
  | { op: "setVariables"; variables: Record<string, string> }
  | { op: "problems" }
  | { op: "undo" }
  | { op: "redo" }
  | { op: "save" }
  | { op: "run"; targetNodeIds?: string[]; fromNodeId?: string }
  | { op: "stop" };

export type ApiEdited = { id?: string; revision: number; problems: ApiProblem[] };

export type ApiRunState = { running: boolean; runId: string | null };

export type ApiProjectState = ApiRunState & {
  dir: string | null;
  workflow: Workflow;
  revision: number;
  saved: boolean;
};

type ApiProblems = { ok: boolean; problems: ApiProblem[] };

export type ApiEditorOp = ApiEditorRequest["op"];

export type ApiEditorResults = {
  open: ApiProjectState;
  read: ApiProjectState;
  replace: ApiEdited;
  addNode: ApiEdited;
  updateNode: ApiEdited;
  removeNodes: ApiEdited;
  addEdge: ApiEdited;
  removeEdges: ApiEdited;
  setSettings: ApiEdited;
  setVariables: ApiEdited;
  problems: ApiProblems;
  undo: ApiEdited;
  redo: ApiEdited;
  save: ApiEdited;
  run: ApiRunState;
  stop: ApiRunState;
};

export type ApiFailure = { ok: false; status: number; error: string; problems?: ApiProblem[] };

export type ApiReply<R> = { ok: true; result: R } | ApiFailure;

export type ApiEditorReply<O extends ApiEditorOp = ApiEditorOp> = ApiReply<ApiEditorResults[O]>;

export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
    readonly problems?: ApiProblem[],
  ) {
    super(message);
  }
}
