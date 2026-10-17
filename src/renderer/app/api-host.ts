import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import type { ApiEditorReply, ApiEditorRequest } from "@/shared/api/types";
import { NO_PROJECT } from "@/shared/errors";
import { useWorkflowStore } from "../store/workflow";

type Handler = (request: ApiEditorRequest) => Promise<ApiEditorReply>;

let editorHandler: Handler | null = null;
const waiters = new Set<() => void>();

export function setEditorApi(handler: Handler): () => void {
  editorHandler = handler;
  for (const wake of waiters) wake();
  return () => {
    if (editorHandler === handler) editorHandler = null;
  };
}

const OPEN_TIMEOUT_MS = 15_000;

function waitForEditor(dir: string): Promise<Handler | null> {
  const ready = () => {
    const s = useWorkflowStore.getState();
    return s.projectDir === dir && s.workflow ? editorHandler : null;
  };
  const now = ready();
  if (now) return Promise.resolve(now);
  return new Promise((resolve) => {
    const finish = (handler: Handler | null) => {
      clearTimeout(timer);
      off();
      waiters.delete(check);
      resolve(handler);
    };
    const check = () => {
      const handler = ready();
      if (handler) finish(handler);
    };
    const timer = setTimeout(() => finish(null), OPEN_TIMEOUT_MS);
    const off = useWorkflowStore.subscribe(check);
    waiters.add(check);
  });
}

export function useApiHost(): void {
  const navigate = useNavigate();
  useEffect(
    () =>
      window.demoflow.onApiRequest(async (request) => {
        if (request.op === "open") {
          void navigate({ to: "/editor", search: { dir: request.dir } });
          const handler = await waitForEditor(request.dir);
          if (!handler)
            return { ok: false, status: 504, error: "The project didn't open in time." };
          return handler({ op: "read" });
        }
        if (!editorHandler) return { ok: false, status: 409, error: NO_PROJECT };
        return editorHandler(request);
      }),
    [navigate],
  );
}
