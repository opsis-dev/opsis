import { create } from "zustand";
import { isPrimaryModifier } from "@/core/menu";
import type { StripSettings } from "@/core/mix";
import { platform } from "@/lib/platform";
import { toggleIds, useUiStore } from "../../store/ui";

type MixerState = {
  clipboard: StripSettings | null;
  anchor: string | null;
  copy: (settings: StripSettings) => void;
  setAnchor: (id: string) => void;
};

export const useMixerState = create<MixerState>()((set) => ({
  clipboard: null,
  anchor: null,
  copy: (clipboard) => set({ clipboard }),
  setAnchor: (anchor) => set({ anchor }),
}));

type PickModifiers = Pick<React.MouseEvent, "shiftKey" | "ctrlKey" | "metaKey">;

function pickedStrips(id: string, order: string[], modifiers: PickModifiers): string[] {
  const { selectedIds } = useUiStore.getState();
  const { anchor } = useMixerState.getState();
  if (modifiers.shiftKey && anchor && order.includes(anchor)) {
    const [a, b] = [order.indexOf(anchor), order.indexOf(id)].toSorted((x, y) => x - y);
    return [...new Set([...selectedIds, ...order.slice(a, b + 1)])];
  }
  if (isPrimaryModifier(modifiers, platform)) return toggleIds(selectedIds, [id]);
  return [id];
}

export function selectStrip(
  id: string,
  order: string[],
  modifiers: PickModifiers,
  sourceNodeId: string | null,
): void {
  const picked = pickedStrips(id, order, modifiers);
  if (!modifiers.shiftKey) useMixerState.getState().setAnchor(id);
  useUiStore.getState().selectPinned(picked, sourceNodeId);
}
