import { create } from "zustand";
import { normalizeWorkflow } from "@/shared/nodes/registry";
import type { Workflow } from "@/shared/schema/workflow";
import { useWorkflowStore } from "../store/workflow";

export type DiskChange = { workflow: Workflow | null; version: string };

export const useDiskChange = create<{
  change: DiskChange | null;
  show: (change: DiskChange | null) => void;
}>()((set) => ({
  change: null,
  show: (change) => set({ change }),
}));

let saving: Promise<boolean> = Promise.resolve(true);

export const whenSaved = (): Promise<void> => saving.then(() => undefined).catch(() => undefined);

export function save(): Promise<boolean> {
  saving = saving
    .catch(() => false)
    .then(async () => {
      const { projectDir, workflow, revision, fileVersion } = useWorkflowStore.getState();
      if (!projectDir || !workflow) return true;
      if (useDiskChange.getState().change) return false;
      const result = await window.demoflow.saveWorkflow(projectDir, workflow, fileVersion);
      if (!result.ok) {
        useDiskChange.getState().show({ workflow: result.workflow, version: result.version });
        return false;
      }
      useWorkflowStore.getState().markSaved(revision);
      useWorkflowStore.getState().setFileVersion(result.version);
      return true;
    });
  return saving;
}

export function loadFromDisk(change: DiskChange): void {
  if (!change.workflow) return;
  const store = useWorkflowStore.getState();
  store.update(() => normalizeWorkflow(change.workflow!));
  const { revision } = useWorkflowStore.getState();
  store.markSaved(revision);
  store.setFileVersion(change.version);
  useDiskChange.getState().show(null);
}

export function keepMine(change: DiskChange): Promise<boolean> {
  useWorkflowStore.getState().setFileVersion(change.version);
  useDiskChange.getState().show(null);
  return save();
}
