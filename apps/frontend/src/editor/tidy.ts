import ElkConstructor from "elkjs/lib/elk-api.js";
import workerUrl from "elkjs/lib/elk-worker.min.js?url";
import { useWorkflowStore, workflowEdits } from "../store/workflow";
import { autoLayout } from "./layout";

const elk = new ElkConstructor({ workerUrl });

export async function tidyLayout(): Promise<void> {
  const wf = useWorkflowStore.getState().workflow;
  if (!wf) return;
  workflowEdits.moveNodes(await autoLayout(elk, wf));
}
