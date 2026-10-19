import { useEffect } from "react";
import { toast } from "sonner";
import type { ProjectFileChange } from "@/shared/ipc";
import { useWorkflowStore } from "../store/workflow";
import { useDiskChange } from "./disk-change-store";
import { loadFromDisk } from "./save";

type EditorFileState = {
  projectDir: string | null;
  fileVersion: string | null;
  unsaved: boolean;
  asking: boolean;
};

function decideDiskChange(
  change: ProjectFileChange,
  editor: EditorFileState,
): "ignore" | "load" | "ask" {
  if (change.dir !== editor.projectDir || change.version === editor.fileVersion) return "ignore";
  return change.workflow && !editor.unsaved && !editor.asking ? "load" : "ask";
}

export function useProjectFileWatch(projectDir: string): void {
  useEffect(() => {
    void window.demoflow.watchProject(projectDir);
    const off = window.demoflow.onProjectFileChanged((change) => {
      const store = useWorkflowStore.getState();
      const decision = decideDiskChange(change, {
        projectDir: store.projectDir,
        fileVersion: store.fileVersion,
        unsaved: store.revision !== store.savedRevision,
        asking: useDiskChange.getState().change !== null,
      });
      if (decision === "ignore") return;
      if (decision === "load") {
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
      useDiskChange.getState().clear();
    };
  }, [projectDir]);
}
