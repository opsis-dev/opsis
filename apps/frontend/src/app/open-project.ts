import type { NavigateFn } from "@tanstack/react-router";
import { useEffect, useEffectEvent, useState } from "react";
import { z } from "zod";
import { commands, type OpenProject } from "@/generated/bindings";
import { reportError } from "@/lib/toast";
import { loadViewState } from "../editor/view-state";
import { useLiveStore } from "../store/live";
import { useUiStore } from "../store/ui";
import { showDocument } from "../store/workflow";

export const PROJECT_DIR_PARAM = z.coerce.string().default("");

export const openEditor = (navigate: NavigateFn, dir: string): void =>
  void navigate({ to: "/editor", search: { dir } });

function openProjectIntoStores(p: OpenProject): void {
  loadViewState(p.dir, p.view);
  showDocument(p.document);
  useUiStore.getState().setSelected([]);
  useUiStore.getState().setReplay(null);
  useLiveStore.getState().reset();
}

export function useOpenProject(dir: string, onFailed: () => void): "opening" | "open" | "failed" {
  const [openedDir, setOpenedDir] = useState<string | null>(null);
  const [failedDir, setFailedDir] = useState<string | null>(null);
  const fail = useEffectEvent((failed: string, e: unknown) => {
    setFailedDir(failed);
    reportError("Couldn't open the project", e);
    onFailed();
  });

  useEffect(() => {
    let cancelled = false;
    commands
      .openProject(dir)
      .then((p) => {
        if (cancelled) return;
        openProjectIntoStores(p);
        setOpenedDir(p.dir);
      })
      .catch((e: unknown) => {
        if (!cancelled) fail(dir, e);
      });
    return () => {
      cancelled = true;
    };
  }, [dir]);

  if (openedDir === dir) return "open";
  return failedDir === dir ? "failed" : "opening";
}
