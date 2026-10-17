import { useEffect } from "react";
import { create } from "zustand";
import { useUiStore } from "../store/ui";

const MAX_LIVE_THUMBS = 6;

type ThumbSlot = { id: string; priority: boolean };

export function claimSlot(
  active: readonly ThumbSlot[],
  id: string,
  priority: boolean,
  max = MAX_LIVE_THUMBS,
): readonly ThumbSlot[] {
  if (active.some((s) => s.id === id)) return active;
  const slot = { id, priority };
  if (active.length < max) return [...active, slot];
  if (!priority) return active;
  const evict = Math.max(
    0,
    active.findIndex((s) => !s.priority),
  );
  return [...active.toSpliced(evict, 1), slot];
}

const useThumbSlots = create<{
  active: readonly ThumbSlot[];
  claim: (id: string, priority: boolean) => void;
  release: (id: string) => void;
}>()((set) => ({
  active: [],
  claim: (id, priority) => set((s) => ({ active: claimSlot(s.active, id, priority) })),
  release: (id) => set((s) => ({ active: s.active.filter((a) => a.id !== id) })),
}));

export function useThumbSlot(nodeId: string, wanted: boolean): boolean {
  const selected = useUiStore((s) => s.selectedIds.includes(nodeId));
  const holding = useThumbSlots((s) => s.active.some((a) => a.id === nodeId));
  useEffect(() => {
    if (!wanted) return;
    const slots = useThumbSlots.getState();
    slots.claim(nodeId, selected);
    return () => slots.release(nodeId);
  }, [wanted, selected, nodeId]);
  return holding;
}
