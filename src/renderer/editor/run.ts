import { toast } from "sonner";
import type { RunRequest } from "@/shared/ipc";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";

export async function startRun(
  opts: Pick<RunRequest, "targetNodeIds" | "fromNodeId"> = {},
): Promise<void> {
  const { projectDir, workflow, revision, markSaved } = useWorkflowStore.getState();
  if (!projectDir || !workflow) return;
  try {
    await window.demoflow.saveWorkflow(projectDir, workflow);
    markSaved(revision);
    useLiveStore.getState().reset();
    const { runId } = await window.demoflow.runWorkflow({ projectDir, workflow, ...opts });
    useRunStore.getState().start(runId);
    useUiStore.getState().setBottomTab("run");
  } catch (e) {
    toast.error("Could not start the run", {
      description: e instanceof Error ? e.message : String(e),
    });
  }
}

export const cancelRun = () => window.demoflow.cancelRun();
