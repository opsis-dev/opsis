import { z } from "zod";
import type {
  addEdgeBody,
  addNodeBody,
  openBody,
  runBody,
  updateNodeBody,
  variablesBody,
} from "#core/api/bodies";
import type { GraphIssue } from "#core/graph";
import { workflowSchema, type Workflow, type WorkflowSettings } from "#core/schema/workflow";

export const API_EDITOR_TIMEOUT_MS = 20_000;

const OPEN_ANSWER_MARGIN_MS = 5_000;

export const API_OPEN_TIMEOUT_MS = API_EDITOR_TIMEOUT_MS - OPEN_ANSWER_MARGIN_MS;

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

type NoBody = Record<never, never>;

type ApiEditorOps = {
  open: { body: z.infer<typeof openBody>; result: ApiProjectState };
  read: { body: NoBody; result: ApiProjectState };
  replace: { body: { workflow: Workflow }; result: ApiEdited };
  addNode: { body: z.infer<typeof addNodeBody>; result: ApiEdited };
  updateNode: { body: { id: string } & z.infer<typeof updateNodeBody>; result: ApiEdited };
  removeNodes: { body: { ids: string[] }; result: ApiEdited };
  addEdge: { body: z.infer<typeof addEdgeBody>; result: ApiEdited };
  removeEdges: { body: { ids: string[] }; result: ApiEdited };
  setSettings: { body: { settings: Partial<WorkflowSettings> }; result: ApiEdited };
  setVariables: { body: { variables: z.infer<typeof variablesBody> }; result: ApiEdited };
  problems: { body: NoBody; result: ApiCheck };
  undo: { body: NoBody; result: ApiEdited };
  redo: { body: NoBody; result: ApiEdited };
  save: { body: NoBody; result: ApiEdited };
  run: { body: z.infer<typeof runBody>; result: ApiRunState };
  stop: { body: NoBody; result: ApiRunState };
};

type ApiEditorOp = keyof ApiEditorOps;

export type ApiEditorRequest = {
  [O in ApiEditorOp]: { op: O } & ApiEditorOps[O]["body"];
}[ApiEditorOp];

type ApiFailure = { ok: false; status: number; error: string; problems?: ApiProblem[] };

export type ApiReply<R> = { ok: true; result: R } | ApiFailure;

export const apiOk = <R>(result: R): ApiReply<R> => ({ ok: true, result });

export const apiFailure = (status: number, error: string): ApiFailure => ({
  ok: false,
  status,
  error,
});

export type ApiEditorReply<O extends ApiEditorOp = ApiEditorOp> = ApiReply<
  ApiEditorOps[O]["result"]
>;
