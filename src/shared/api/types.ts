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

export type ApiEditorReply =
  | { ok: true; result: unknown }
  | { ok: false; status: number; error: string; problems?: ApiProblem[] };

export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
    readonly problems?: ApiProblem[],
  ) {
    super(message);
  }
}
