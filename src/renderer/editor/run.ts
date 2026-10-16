import { toast } from "sonner";
import type { RunRequest } from "@/shared/ipc";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useWorkflowStore } from "../store/workflow";
import { showPanel } from "./panels";
import { save } from "./save";

type RunTargets = Pick<RunRequest, "targetNodeIds" | "fromNodeId">;

export async function beginRun(opts: RunTargets = {}): Promise<string> {
  const { projectDir } = useWorkflowStore.getState();
  if (!projectDir || !useWorkflowStore.getState().workflow) throw new Error("No project is open.");
  if (!(await save())) throw new Error("Choose which version of the project to keep first.");
  const workflow = useWorkflowStore.getState().workflow!;
  useLiveStore.getState().reset();
  const { runId } = await window.demoflow.runWorkflow({ projectDir, workflow, ...opts });
  useRunStore.getState().start(runId);
  showPanel("run");
  return runId;
}

export async function startRun(opts: RunTargets = {}): Promise<void> {
  try {
    await beginRun(opts);
  } catch (e) {
    toast.error("Could not start the run", {
      description: e instanceof Error ? e.message : String(e),
    });
  }
}

export const cancelRun = () => window.demoflow.cancelRun();
