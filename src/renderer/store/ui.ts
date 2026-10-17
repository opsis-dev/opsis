import { create } from "zustand";
import { pick } from "@/shared/pick";

export const singleSelected = (ids: string[]): string | null => (ids.length === 1 ? ids[0] : null);

export function toggleIds(current: string[], ids: string[]): string[] {
  const out = new Set(current);
  for (const id of ids) {
    if (out.has(id)) out.delete(id);
    else out.add(id);
  }
  return [...out];
}

export const REPLAY_TABS = ["replay", "trace"] as const;
type ReplayTab = (typeof REPLAY_TABS)[number];

type UiState = {
  selectedIds: string[];
  activeIds: string[];
  paletteOpen: boolean;
  replayNodeId: string | null;
  replayTab: ReplayTab;
  pinnedSource: string | null;
  reveal: { id: string; nonce: number } | null;
  revealNode: (id: string) => void;
  setSelected: (ids: string[]) => void;
  selectPinned: (ids: string[], source: string | null) => void;
  pinSource: (source: string | null) => void;
  setActive: (ids: string[]) => void;
  setPaletteOpen: (open: boolean) => void;
  setReplay: (nodeId: string | null, tab?: ReplayTab) => void;
  setReplayTab: (tab: ReplayTab) => void;
};

const sameIds = (a: string[], b: string[]) =>
  a.length === b.length && a.every((x, i) => x === b[i]);

export const useUiStore = create<UiState>()((set) => ({
  selectedIds: [],
  activeIds: [],
  paletteOpen: false,
  replayNodeId: null,
  replayTab: "replay",
  pinnedSource: null,
  reveal: null,
  revealNode: (id) => set((s) => ({ reveal: { id, nonce: (s.reveal?.nonce ?? 0) + 1 } })),
  setSelected: (selectedIds) => set({ selectedIds, pinnedSource: null }),
  selectPinned: (selectedIds, pinnedSource) => set({ selectedIds, pinnedSource }),
  pinSource: (pinnedSource) => set({ pinnedSource }),
  setActive: (activeIds) => set((s) => (sameIds(s.activeIds, activeIds) ? {} : { activeIds })),
  setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  setReplay: (replayNodeId, replayTab = "replay") => set({ replayNodeId, replayTab }),
  setReplayTab: (replayTab) => set({ replayTab }),
}));

export const SELECTION_KEYS = ["selectedIds", "activeIds", "pinnedSource"] as const;

export const UI_LOCAL_KEYS = ["paletteOpen", "replayNodeId", "replayTab", "reveal"] as const;

export type Selection = Pick<UiState, (typeof SELECTION_KEYS)[number]>;

export const selection = (): Selection => pick(useUiStore.getState(), SELECTION_KEYS);
