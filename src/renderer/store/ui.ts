import { create } from "zustand";
import { VARIANTS, type Variant } from "@/shared/nodes/output";
import { readPref, writePref } from "./view-prefs";

export type PreviewQuality = "fast" | "exact";
const isQuality = (v: unknown) => v === "fast" || v === "exact";

export const storedQuality = (): PreviewQuality => readPref("previewQuality", "fast", isQuality);

export type PreviewShape = "project" | Variant;
const isShape = (v: unknown) => v === "project" || VARIANTS.includes(v as Variant);

export const storedShape = (): PreviewShape => readPref("previewShape", "project", isShape);

type UiState = {
  selectedIds: string[];
  activeIds: string[];
  seekTo: number | null;
  paletteOpen: boolean;
  replayNodeId: string | null;
  replayTab: ReplayTab;
  pendingProjectAction: "new" | "open" | null;
  setPendingProjectAction: (action: "new" | "open" | null) => void;
  previewQuality: PreviewQuality;
  setPreviewQuality: (quality: PreviewQuality) => void;
  previewShape: PreviewShape;
  setPreviewShape: (shape: PreviewShape) => void;
  pinnedSource: string | null;
  reveal: { id: string; nonce: number } | null;
  revealNode: (id: string) => void;
  setSelected: (ids: string[]) => void;
  selectPinned: (ids: string[], source: string | null) => void;
  pinSource: (source: string | null) => void;
  setActive: (ids: string[]) => void;
  seek: (frame: number | null) => void;
  setPaletteOpen: (open: boolean) => void;
  setReplay: (nodeId: string | null, tab?: ReplayTab) => void;
  setReplayTab: (tab: ReplayTab) => void;
};

export type ReplayTab = "replay" | "trace";

export const useUiStore = create<UiState>()((set) => ({
  selectedIds: [],
  activeIds: [],
  seekTo: null,
  paletteOpen: false,
  replayNodeId: null,
  replayTab: "replay",
  pendingProjectAction: null,
  setPendingProjectAction: (pendingProjectAction) => set({ pendingProjectAction }),
  previewQuality: storedQuality(),
  setPreviewQuality: (previewQuality) => {
    set({ previewQuality });
    writePref("previewQuality", previewQuality);
  },
  previewShape: storedShape(),
  setPreviewShape: (previewShape) => {
    set({ previewShape });
    writePref("previewShape", previewShape);
  },

  pinnedSource: null,
  reveal: null,
  revealNode: (id) => set((s) => ({ reveal: { id, nonce: (s.reveal?.nonce ?? 0) + 1 } })),
  setSelected: (selectedIds) => set({ selectedIds, pinnedSource: null }),
  selectPinned: (selectedIds, pinnedSource) => set({ selectedIds, pinnedSource }),
  pinSource: (pinnedSource) => set({ pinnedSource }),
  setActive: (activeIds) => set((s) => (sameIds(s.activeIds, activeIds) ? {} : { activeIds })),
  seek: (seekTo) => set({ seekTo }),
  setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  setReplay: (replayNodeId, replayTab = "replay") => set({ replayNodeId, replayTab }),
  setReplayTab: (replayTab) => set({ replayTab }),
}));

const sameIds = (a: string[], b: string[]) =>
  a.length === b.length && a.every((x, i) => x === b[i]);
