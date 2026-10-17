import { normalizeWorkflow } from "@/shared/schema/workflow";
import { useWorkflowStore } from "../store/workflow";
import { useDiskChange, type DiskChange } from "./disk-change-store";
import { reportError } from "@/renderer/lib/toast";

let saving: Promise<boolean> = Promise.resolve(true);

export const whenSaved = (): Promise<boolean> => saving.catch(() => false);

export function save(): Promise<boolean> {
  saving = whenSaved().then(async () => {
    const { projectDir, workflow, revision, fileVersion } = useWorkflowStore.getState();
    if (!projectDir || !workflow) return true;
    if (useDiskChange.getState().change) return false;
    const result = await window.demoflow.saveWorkflow(projectDir, workflow, fileVersion);
    if (!result.ok) {
      useDiskChange.getState().show({ workflow: result.workflow, version: result.version });
      return false;
    }
    useWorkflowStore.getState().markSaved(revision, result.version);
    return true;
  });
  return saving;
}

export const saveWithToast = (): Promise<boolean> =>
  save().catch((e: unknown) => {
    reportError("Save failed", e);
    return false;
  });

export function loadFromDisk({ workflow, version }: DiskChange): void {
  if (!workflow) return;
  const store = useWorkflowStore.getState();
  store.update(() => normalizeWorkflow(workflow));
  store.markSaved(useWorkflowStore.getState().revision, version);
  useDiskChange.getState().show(null);
}

export function keepMine(change: DiskChange): Promise<boolean> {
  useWorkflowStore.getState().setFileVersion(change.version);
  useDiskChange.getState().show(null);
  return save();
}
