import type { Workflow } from "@/generated/bindings";
import { NO_PROJECT } from "@/lib/errors";
import { useWorkflowStore } from "../store/workflow";

function requireWorkflow(state: { workflow: Workflow | null }): Workflow {
  if (!state.workflow) throw new Error(NO_PROJECT);
  return state.workflow;
}

export function useRequiredWorkflow(): Workflow {
  return useWorkflowStore(requireWorkflow);
}
