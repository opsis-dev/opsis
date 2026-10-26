import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { create } from "zustand";
import { API_OPEN_TIMEOUT_MS, type ApiEditorReply, type ApiEditorRequest } from "@/core/api";
import { NO_PROJECT } from "@/core/errors";
import { whenReady } from "@/core/timing";
import { useWorkflowStore } from "../store/workflow";
import { openEditor, useOpenFailure } from "./open-project";

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

type Opened = { handler: Handler } | { error: string };

function openedEditor(dir: string): Opened | null {
  const failure = useOpenFailure.getState();
  if (failure?.dir === dir) return { error: failure.error };
  const handler = editorWithProject(dir);
  return handler ? { handler } : null;
}

function onEditorChange(onChange: () => void): () => void {
  const offWorkflow = useWorkflowStore.subscribe(onChange);
  const offEditor = useEditorApi.subscribe(onChange);
  const offFailure = useOpenFailure.subscribe(onChange);
  return () => {
    offWorkflow();
    offEditor();
    offFailure();
  };
}

const waitForEditor = (dir: string): Promise<Opened | null> =>
  whenReady(() => openedEditor(dir), onEditorChange, API_OPEN_TIMEOUT_MS);

export function useApiHost(): void {
  const navigate = useNavigate();
  useEffect(
    () =>
      window.opsis.onApiRequest(async (request) => {
        if (request.op === "open") {
          useOpenFailure.setState(null, true);
          openEditor(navigate, request.dir);
          const opened = await waitForEditor(request.dir);
          if (!opened) return { ok: false, status: 504, error: "The project didn't open in time." };
          if ("error" in opened)
            return { ok: false, status: 422, error: `Couldn't open the project. ${opened.error}` };
          return opened.handler({ op: "read" });
        }
        const handler = useEditorApi.getState().handler;
        if (!handler) return { ok: false, status: 409, error: NO_PROJECT };
        return handler(request);
      }),
    [navigate],
  );
}
