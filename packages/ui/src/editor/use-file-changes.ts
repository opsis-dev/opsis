import { useEffect } from "react";
import { useMedia } from "../store/media";
import { useProjectFiles } from "../store/project-files";
import { useWorkflowStore } from "../store/workflow";

export function useFileChanges(): void {
  useEffect(
    () =>
      window.opsis.onProjectFilesChanged((change) => {
        if (change.dir !== useWorkflowStore.getState().projectDir) return;
        useProjectFiles.getState().changed(change.files);
        useMedia.getState().reloadChanged(Object.keys(change.files));
      }),
    [],
  );
}
