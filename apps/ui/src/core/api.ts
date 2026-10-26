import type { EditorRequest, Workflow } from "@/generated/bindings";

export const API_EDITOR_TIMEOUT_MS = 20_000;

const OPEN_ANSWER_MARGIN_MS = 5_000;

export const API_OPEN_TIMEOUT_MS = API_EDITOR_TIMEOUT_MS - OPEN_ANSWER_MARGIN_MS;

export type ApiEditorRequest = EditorRequest;

export type ApiEdited = { id?: string; revision: number; workflow: Workflow };

export type ApiRunState = { running: boolean; runId: string | null };

export type ApiProjectState = ApiRunState & {
  dir: string | null;
  workflow: Workflow;
  revision: number;
  saved: boolean;
};

type ApiFailure = { ok: false; status: number; error: string };

export type ApiReply<R> = { ok: true; result: R } | ApiFailure;

export type ApiEditorReply = ApiReply<unknown>;

export const apiOk = <R>(result: R): ApiReply<R> => ({ ok: true, result });

export const apiFailure = (status: number, error: string): ApiFailure => ({
  ok: false,
  status,
  error,
});
