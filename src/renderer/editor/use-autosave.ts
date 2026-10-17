import { useEffect } from "react";
import { useWorkflowStore } from "../store/workflow";
import { save, saveWithToast } from "./save";

const AUTOSAVE_DELAY_MS = 800;

export function useAutosave(): void {
  const revision = useWorkflowStore((s) => s.revision);
  const saved = useWorkflowStore((s) => s.savedRevision);
  useEffect(() => {
    window.demoflow.reportUnsaved(revision !== saved);
  }, [revision, saved]);
  useEffect(() => {
    const off = window.demoflow.onFlushSave(() => save().then(() => undefined));
    return () => {
      off();
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
