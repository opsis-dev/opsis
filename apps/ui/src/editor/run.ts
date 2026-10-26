import type { RunTargets } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { showPanel } from "./panels";

export const startRun = (targets: RunTargets = {}): Promise<void> =>
  attempt(async () => {
    useLiveStore.getState().reset();
    const { runId } = await window.opsis.runWorkflow(targets);
    useRunStore.getState().start(runId);
    showPanel("run");
  }, "Couldn't start the run");
