import { useEffect } from "react";
import { on } from "@/app/tauri";
import { commands, events } from "@/generated/bindings";
import { commitHeldDrafts, useHeldDrafts } from "@/lib/use-commit-draft";
import { saveWithToast } from "./save";

export function useHandOverDrafts(): void {
  const drafts = useHeldDrafts();
  useEffect(() => {
    void commands.reportDrafts(drafts);
  }, [drafts]);
  useEffect(() => {
    const off = on(events.flushSave, () => {
      try {
        commitHeldDrafts();
      } finally {
        void commands.saveFlushed();
      }
    });
    return () => {
      off();
      commitHeldDrafts();
      void saveWithToast();
      void commands.reportDrafts(false);
    };
  }, []);
}
