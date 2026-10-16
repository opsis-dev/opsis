import { useEffect } from "react";
import { toast } from "sonner";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/renderer/components/ui/alert-dialog";
import { useWorkflowStore } from "../store/workflow";
import { keepMine, loadFromDisk, useDiskChange } from "./save";

export function useProjectFileWatch(projectDir: string): void {
  useEffect(() => {
    void window.demoflow.watchProject(projectDir);
    const off = window.demoflow.onProjectFileChanged((change) => {
      const store = useWorkflowStore.getState();
      if (change.dir !== store.projectDir || change.version === store.fileVersion) return;
      const pending = useDiskChange.getState().change;
      if (store.revision === store.savedRevision && !pending) {
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

export function DiskChangeDialog() {
  const change = useDiskChange((s) => s.change);
  const readable = !!change?.workflow;
  return (
    <AlertDialog open={!!change}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>The project file changed on disk</AlertDialogTitle>
          <AlertDialogDescription>
            {readable
              ? "Something outside Demoflow changed this project while you had unsaved edits. Load the file to use its version (Undo brings yours back), or keep yours to save over it."
              : "Something outside Demoflow changed this project, and the file isn't a valid project right now. Keep yours to save over it, or wait and fix the file first."}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          {readable ? (
            <AlertDialogCancel onClick={() => change && loadFromDisk(change)}>
              Load the file
            </AlertDialogCancel>
          ) : (
            <AlertDialogCancel onClick={() => useDiskChange.getState().show(null)}>
              Wait
            </AlertDialogCancel>
          )}
          <AlertDialogAction onClick={() => change && void keepMine(change)}>
            Keep mine
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
