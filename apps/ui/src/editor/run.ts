import type { RunTargets } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { useLiveStore } from "../store/live";
import { showPanel } from "./panels";

export const startRun = (targets: RunTargets = {}): Promise<void> =>
  attempt(async () => {
    useLiveStore.getState().reset();
    await window.opsis.runWorkflow(targets);
    showPanel("run");
  }, "Couldn't start the run");
