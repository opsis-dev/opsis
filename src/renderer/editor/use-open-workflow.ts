import type { Workflow } from "@/shared/schema/workflow";
import { useWorkflowStore } from "../store/workflow";

export function openWorkflow(state: { workflow: Workflow | null }): Workflow {
  if (!state.workflow) throw new Error("No project is open.");
  return state.workflow;
}

export const useOpenWorkflow = (): Workflow => useWorkflowStore(openWorkflow);
