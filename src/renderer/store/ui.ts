import { create } from "zustand";

export type PreviewQuality = "fast" | "exact";
const QUALITY_KEY = "demoflow.previewQuality";

function storedQuality(): PreviewQuality {
  try {
    return window.localStorage.getItem(QUALITY_KEY) === "exact" ? "exact" : "fast";
  } catch {
    return "fast";
  }
}

type UiState = {
  selectedIds: string[];
  activeIds: string[];
  seekTo: number | null;
  paletteOpen: boolean;
  bottomTab: "preview" | "run";
  replayNodeId: string | null;
  replayTab: ReplayTab;
  previewQuality: PreviewQuality;
  setPreviewQuality: (quality: PreviewQuality) => void;
  setSelected: (ids: string[]) => void;
  setActive: (ids: string[]) => void;
  seek: (frame: number | null) => void;
  setPaletteOpen: (open: boolean) => void;
  setBottomTab: (tab: "preview" | "run") => void;
  setReplay: (nodeId: string | null, tab?: ReplayTab) => void;
  setReplayTab: (tab: ReplayTab) => void;
};

export type ReplayTab = "replay" | "trace";

export const useUiStore = create<UiState>()((set) => ({
  selectedIds: [],
  activeIds: [],
  seekTo: null,
  paletteOpen: false,
  bottomTab: "preview",
  replayNodeId: null,
  replayTab: "replay",
  previewQuality: storedQuality(),
  setPreviewQuality: (previewQuality) => {
    set({ previewQuality });
    try {
      window.localStorage.setItem(QUALITY_KEY, previewQuality);
    } catch {
      return;
    }
  },
  setSelected: (selectedIds) => set({ selectedIds }),
  setActive: (activeIds) => set((s) => (sameIds(s.activeIds, activeIds) ? {} : { activeIds })),
  seek: (seekTo) => set({ seekTo }),
  setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  setBottomTab: (bottomTab) => set({ bottomTab }),
  setReplay: (replayNodeId, replayTab = "replay") => set({ replayNodeId, replayTab }),
  setReplayTab: (replayTab) => set({ replayTab }),
}));

const sameIds = (a: string[], b: string[]) =>
  a.length === b.length && a.every((x, i) => x === b[i]);
