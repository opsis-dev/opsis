import { commands, type RunTargets } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { useLiveStore } from "../store/live";
import { showPanel } from "./panels";

export function startRun(targets: RunTargets = {}): Promise<void> {
  return attempt(async () => {
    useLiveStore.getState().reset();
    await commands.runWorkflow(targets);
    showPanel("run");
  }, "Couldn't start the run");
}
