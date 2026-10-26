import { useEffect } from "react";
import { commitHeldDrafts, useHeldDrafts } from "@/lib/use-commit-draft";
import { saveWithToast } from "./save";

export function useHandOverDrafts(): void {
  const drafts = useHeldDrafts();
  useEffect(() => {
    window.opsis.reportDrafts(drafts);
  }, [drafts]);
  useEffect(() => {
    const off = window.opsis.onFlushSave(async () => commitHeldDrafts());
    return () => {
      off();
      commitHeldDrafts();
      void saveWithToast();
      window.opsis.reportDrafts(false);
    };
  }, []);
}
