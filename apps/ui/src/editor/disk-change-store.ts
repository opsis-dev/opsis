import type { Workflow } from "@opsis/core/schema/workflow";
import { create } from "zustand";

export type DiskChange = { workflow: Workflow | null; version: string };

export const useDiskChange = create<{
  change: DiskChange | null;
  show: (change: DiskChange) => void;
  clear: () => void;
}>()((set) => ({
  change: null,
  show: (change) => set({ change }),
  clear: () => set({ change: null }),
}));
