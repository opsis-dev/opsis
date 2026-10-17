import { useEffect, useEffectEvent, useState } from "react";
import { toast } from "sonner";
import type { OpenProject } from "@/shared/ipc";
import { errorMessage } from "@/shared/errors";
import { whenSaved } from "../editor/save";
import { loadViewState } from "../editor/view-state";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";

function openProjectIntoStores(p: OpenProject): void {
  loadViewState(p.dir);
  useWorkflowStore.getState().load(p.dir, p.workflow, p.version);
  useRunStore.getState().seed(p.lastRun);
  useUiStore.getState().setSelected([]);
  useUiStore.getState().setReplay(null);
  useLiveStore.getState().reset();
  void useSecretsStore.getState().refresh(p.dir);
  void window.demoflow.runStatus().then((s) => {
    if (s.runId && s.projectDir === p.dir) useRunStore.getState().resume(s.progress);
  });
}

export function useOpenProject(dir: string, onFailed: () => void): "opening" | "open" | "failed" {
  const loadedDir = useWorkflowStore((s) => (s.workflow ? s.projectDir : null));
  const [failed, setFailed] = useState(false);
  const fail = useEffectEvent((e: unknown) => {
    setFailed(true);
    toast.error("Could not open the project", { description: errorMessage(e) });
    onFailed();
  });

  useEffect(() => {
    let cancelled = false;
    whenSaved()
      .then(() => window.demoflow.openProject(dir))
      .then((p) => {
        if (!cancelled && p) openProjectIntoStores(p);
      })
      .catch((e: unknown) => {
        if (!cancelled) fail(e);
      });
    return () => {
      cancelled = true;
    };
  }, [dir]);

  if (loadedDir === dir) return "open";
  return failed ? "failed" : "opening";
}
