import { NO_PROJECT } from "@opsis/core/errors";
import type { Workflow } from "@opsis/core/schema/workflow";
import { useWorkflowStore } from "../store/workflow";

export function requireWorkflow(state: { workflow: Workflow | null }): Workflow {
  if (!state.workflow) throw new Error(NO_PROJECT);
  return state.workflow;
}

export const useRequiredWorkflow = (): Workflow => useWorkflowStore(requireWorkflow);
