import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import type { ApiEditorReply, ApiEditorRequest } from "@/shared/api/types";
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

function waitForProject(dir: string): Promise<boolean> {
  const ready = () => {
    const s = useWorkflowStore.getState();
    return s.projectDir === dir && !!s.workflow && !!editorHandler;
  };
  if (ready()) return Promise.resolve(true);
  return new Promise((resolve) => {
    const finish = (ok: boolean) => {
      clearTimeout(timer);
      off();
      waiters.delete(check);
      setTimeout(() => resolve(ok), 0);
    };
    const check = () => {
      if (ready()) finish(true);
    };
    const timer = setTimeout(() => finish(false), OPEN_TIMEOUT_MS);
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
          if (!(await waitForProject(request.dir)))
            return { ok: false, status: 504, error: "The project didn't open in time." };
          return editorHandler!({ op: "read" });
        }
        if (!editorHandler) return { ok: false, status: 409, error: "No project is open." };
        return editorHandler(request);
      }),
    [navigate],
  );
}
