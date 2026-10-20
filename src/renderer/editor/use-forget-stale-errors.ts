import { useEffect } from "react";
import { editedNodes } from "@/shared/workflow-diff";
import { useRunStore } from "../store/run";
import { useWorkflowStore } from "../store/workflow";

export function useForgetStaleErrors(): void {
  useEffect(
    () =>
      useWorkflowStore.subscribe((next, prev) => {
        if (!next.workflow || !prev.workflow || next.workflow === prev.workflow) return;
        if (next.projectDir !== prev.projectDir) return;
        const edited = editedNodes(prev.workflow, next.workflow);
        if (edited.length) useRunStore.getState().forgetErrors(edited);
      }),
    [],
  );
}
