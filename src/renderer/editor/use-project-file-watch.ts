import { useEffect } from "react";
import { toast } from "sonner";
import { useWorkflowStore } from "../store/workflow";
import { useDiskChange } from "./disk-change-store";
import { loadFromDisk } from "./save";

export function useProjectFileWatch(projectDir: string): void {
  useEffect(() => {
    void window.demoflow.watchProject(projectDir);
    const off = window.demoflow.onProjectFileChanged((change) => {
      const store = useWorkflowStore.getState();
      if (change.dir !== store.projectDir || change.version === store.fileVersion) return;
      const pending = useDiskChange.getState().change;
      if (change.workflow && store.revision === store.savedRevision && !pending) {
        loadFromDisk(change);
        toast.info("The project file changed on disk, so the editor loaded it.", {
          description: "Undo brings back the previous version.",
        });
        return;
      }
      useDiskChange.getState().show({ workflow: change.workflow, version: change.version });
    });
    return () => {
      off();
      void window.demoflow.watchProject(null);
      useDiskChange.getState().show(null);
    };
  }, [projectDir]);
}
