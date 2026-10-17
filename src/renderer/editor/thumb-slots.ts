import { useEffect } from "react";
import { create } from "zustand";
import { useUiStore } from "../store/ui";

const MAX_LIVE_THUMBS = 6;

const useThumbSlots = create<{
  active: string[];
  claim: (id: string, priority: boolean) => boolean;
  release: (id: string) => void;
}>()((set, get) => ({
  active: [],
  claim: (id, priority) => {
    const { active } = get();
    if (active.includes(id)) return true;
    if (active.length < MAX_LIVE_THUMBS) {
      set({ active: [...active, id] });
      return true;
    }
    if (!priority) return false;
    set({ active: [...active.slice(1), id] });
    return true;
  },
  release: (id) => set((s) => ({ active: s.active.filter((a) => a !== id) })),
}));

export function useThumbSlot(nodeId: string, wanted: boolean): boolean {
  const selected = useUiStore((s) => s.selectedIds.includes(nodeId));
  const holding = useThumbSlots((s) => s.active.includes(nodeId));
  useEffect(() => {
    if (!wanted) return;
    const slots = useThumbSlots.getState();
    slots.claim(nodeId, selected);
    return () => slots.release(nodeId);
  }, [wanted, selected, nodeId]);
  return holding;
}
