import { NO_PROJECT } from "@/shared/errors";
import type { Workflow } from "@/shared/schema/workflow";
import { useWorkflowStore } from "../store/workflow";

export function requireWorkflow(state: { workflow: Workflow | null }): Workflow {
  if (!state.workflow) throw new Error(NO_PROJECT);
  return state.workflow;
}

export const useRequiredWorkflow = (): Workflow => useWorkflowStore(requireWorkflow);
