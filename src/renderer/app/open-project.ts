import { useEffect, useEffectEvent, useState } from "react";
import type { OpenProject } from "@/shared/ipc";
import { whenSaved } from "../editor/save";
import { loadViewState } from "../editor/view-state";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { reportError } from "@/renderer/lib/toast";

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
  const [failedDir, setFailedDir] = useState<string | null>(null);
  const fail = useEffectEvent((failed: string, e: unknown) => {
    setFailedDir(failed);
    reportError("Could not open the project", e);
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
        if (!cancelled) fail(dir, e);
      });
    return () => {
      cancelled = true;
    };
  }, [dir]);

  if (loadedDir === dir) return "open";
  return failedDir === dir ? "failed" : "opening";
}
