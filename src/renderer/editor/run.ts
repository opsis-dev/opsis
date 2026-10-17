import { toast } from "sonner";
import type { RunRequest } from "@/shared/ipc";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useWorkflowStore } from "../store/workflow";
import { showPanel } from "./panels";
import { save } from "./save";
import { errorMessage } from "@/shared/errors";

type RunTargets = Pick<RunRequest, "targetNodeIds" | "fromNodeId">;

const NO_PROJECT = "No project is open.";

export async function beginRun(opts: RunTargets = {}): Promise<string> {
  if (!(await save())) throw new Error("Choose which version of the project to keep first.");
  const { projectDir, workflow } = useWorkflowStore.getState();
  if (!projectDir || !workflow) throw new Error(NO_PROJECT);
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
      description: errorMessage(e),
    });
  }
}
