import type { Workflow } from "@/shared/schema/workflow";
import { useWorkflowStore } from "../store/workflow";

export function useOpenWorkflow(): Workflow {
  const workflow = useWorkflowStore((s) => s.workflow);
  if (!workflow) throw new Error("No project is open.");
  return workflow;
}
