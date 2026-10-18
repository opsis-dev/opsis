import { z } from "zod";
import { workflowSchema, type Workflow, type WorkflowSettings } from "../schema/workflow";
import type { GraphIssue } from "../graph";
import type {
  addEdgeBody,
  addNodeBody,
  openBody,
  runBody,
  updateNodeBody,
  variablesBody,
} from "./bodies";

export const apiProblemSchema = z.object({
  level: z.enum(["error", "warning"]),
  message: z.string(),
  nodeId: z.string().optional(),
  edgeId: z.string().optional(),
  path: z.string().optional().meta({ description: "Where in the file, for format problems" }),
}) satisfies z.ZodType<ApiProblem>;
export type ApiProblem = GraphIssue & { path?: string };

const problemList = z.array(apiProblemSchema);

export const apiCheckSchema = z.object({
  ok: z.boolean().meta({ description: "True when there are no errors" }),
  problems: problemList,
});
export type ApiCheck = z.infer<typeof apiCheckSchema>;

export const apiEditedSchema = z.object({
  id: z.string().optional().meta({ description: "The new node's or connection's id" }),
  revision: z.number().int(),
  problems: problemList,
});
export type ApiEdited = z.infer<typeof apiEditedSchema>;

export const apiRunStateSchema = z.object({ running: z.boolean(), runId: z.string().nullable() });
export type ApiRunState = z.infer<typeof apiRunStateSchema>;

export const apiStatusSchema = z.object({
  app: z.string(),
  version: z.string(),
  project: z.object({ dir: z.string(), name: z.string() }).nullable(),
  running: z.boolean(),
});

export const apiErrorSchema = z.object({ error: z.string(), problems: problemList.optional() });

export const apiProjectStateSchema = apiRunStateSchema.extend({
  dir: z.string().nullable(),
  workflow: workflowSchema,
  revision: z.number().int().meta({ description: "Goes up with every edit" }),
  saved: z.boolean(),
});
export type ApiProjectState = z.infer<typeof apiProjectStateSchema>;

export type ApiEditorRequest =
  | ({ op: "open" } & z.infer<typeof openBody>)
  | { op: "read" }
  | { op: "replace"; workflow: Workflow }
  | ({ op: "addNode" } & z.infer<typeof addNodeBody>)
  | ({ op: "updateNode"; id: string } & z.infer<typeof updateNodeBody>)
  | { op: "removeNodes"; ids: string[] }
  | ({ op: "addEdge" } & z.infer<typeof addEdgeBody>)
  | { op: "removeEdges"; ids: string[] }
  | { op: "setSettings"; settings: Partial<WorkflowSettings> }
  | { op: "setVariables"; variables: z.infer<typeof variablesBody> }
  | { op: "problems" }
  | { op: "undo" }
  | { op: "redo" }
  | { op: "save" }
  | ({ op: "run" } & z.infer<typeof runBody>)
  | { op: "stop" };

type ApiEditorOp = ApiEditorRequest["op"];

type ApiEditorResults = {
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
  problems: ApiCheck;
  undo: ApiEdited;
  redo: ApiEdited;
  save: ApiEdited;
  run: ApiRunState;
  stop: ApiRunState;
};

type ApiFailure = { ok: false; status: number; error: string; problems?: ApiProblem[] };

export type ApiReply<R> = { ok: true; result: R } | ApiFailure;

export const apiOk = <R>(result: R): ApiReply<R> => ({ ok: true, result });

export const apiFailure = (status: number, error: string): ApiFailure => ({
  ok: false,
  status,
  error,
});

export type ApiEditorReply<O extends ApiEditorOp = ApiEditorOp> = ApiReply<ApiEditorResults[O]>;
