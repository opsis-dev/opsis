import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { create } from "zustand";
import {
  API_OPEN_TIMEOUT_MS,
  type ApiEditorReply,
  type ApiEditorRequest,
} from "@/shared/api/types";
import { NO_PROJECT } from "@/shared/errors";
import { whenReady } from "@/shared/timing";
import { useWorkflowStore } from "../store/workflow";
import { openEditor } from "./open-project";

type Handler = (request: ApiEditorRequest) => Promise<ApiEditorReply>;

const useEditorApi = create<{ handler: Handler | null }>()(() => ({ handler: null }));

export function setEditorApi(handler: Handler): () => void {
  useEditorApi.setState({ handler });
  return () => {
    if (useEditorApi.getState().handler === handler) useEditorApi.setState({ handler: null });
  };
}

function editorWithProject(dir: string): Handler | null {
  const { projectDir, workflow } = useWorkflowStore.getState();
  return projectDir === dir && workflow ? useEditorApi.getState().handler : null;
}

function onEditorChange(onChange: () => void): () => void {
  const offWorkflow = useWorkflowStore.subscribe(onChange);
  const offEditor = useEditorApi.subscribe(onChange);
  return () => {
    offWorkflow();
    offEditor();
  };
}

const waitForEditor = (dir: string): Promise<Handler | null> =>
  whenReady(() => editorWithProject(dir), onEditorChange, API_OPEN_TIMEOUT_MS);

export function useApiHost(): void {
  const navigate = useNavigate();
  useEffect(
    () =>
      window.demoflow.onApiRequest(async (request) => {
        if (request.op === "open") {
          openEditor(navigate, request.dir);
          const handler = await waitForEditor(request.dir);
          if (!handler)
            return { ok: false, status: 504, error: "The project didn't open in time." };
          return handler({ op: "read" });
        }
        const handler = useEditorApi.getState().handler;
        if (!handler) return { ok: false, status: 409, error: NO_PROJECT };
        return handler(request);
      }),
    [navigate],
  );
}
