import { NO_PROJECT } from "@opsis/core/errors";
import type { RunRequest } from "@opsis/core/ipc";
import { attempt } from "@/lib/toast";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useWorkflowStore } from "../store/workflow";
import { showPanel } from "./panels";
import { save, VERSION_UNCHOSEN } from "./save";

type RunTargets = Pick<RunRequest, "targetNodeIds" | "fromNodeId">;

export async function beginRun(opts: RunTargets = {}): Promise<string> {
  if (!(await save())) throw new Error(VERSION_UNCHOSEN);
  const { projectDir, workflow } = useWorkflowStore.getState();
  if (!projectDir || !workflow) throw new Error(NO_PROJECT);
  useLiveStore.getState().reset();
  const { runId } = await window.opsis.runWorkflow({ projectDir, workflow, ...opts });
  useRunStore.getState().start(runId);
  showPanel("run");
  return runId;
}

export const startRun = (opts: RunTargets = {}): Promise<void> =>
  attempt(() => beginRun(opts), "Couldn't start the run");
