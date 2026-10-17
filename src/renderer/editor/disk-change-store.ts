import { create } from "zustand";
import type { Workflow } from "@/shared/schema/workflow";

export type DiskChange = { workflow: Workflow | null; version: string };

export const useDiskChange = create<{
  change: DiskChange | null;
  show: (change: DiskChange | null) => void;
}>()((set) => ({
  change: null,
  show: (change) => set({ change }),
}));
