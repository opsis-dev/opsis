import { useEffect } from "react";
import { commitHeldDrafts, useHeldDrafts } from "@/renderer/lib/use-commit-draft";
import { useWorkflowStore } from "../store/workflow";
import { save, saveWithToast } from "./save";

const AUTOSAVE_DELAY_MS = 800;

export function useAutosave(): void {
  const revision = useWorkflowStore((s) => s.revision);
  const saved = useWorkflowStore((s) => s.savedRevision);
  const drafts = useHeldDrafts();
  useEffect(() => {
    window.demoflow.reportUnsaved(revision !== saved || drafts);
  }, [revision, saved, drafts]);
  useEffect(() => {
    const off = window.demoflow.onFlushSave(() => {
      commitHeldDrafts();
      return save().then(() => undefined);
    });
    return () => {
      off();
      commitHeldDrafts();
      const state = useWorkflowStore.getState();
      if (state.revision !== state.savedRevision) void saveWithToast();
      window.demoflow.reportUnsaved(false);
    };
  }, []);
  useEffect(() => {
    if (revision === saved) return;
    const t = setTimeout(() => void saveWithToast(), AUTOSAVE_DELAY_MS);
    return () => clearTimeout(t);
  }, [revision, saved]);
}
