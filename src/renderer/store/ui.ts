import { create } from "zustand";
import { VARIANTS } from "@/shared/nodes/output";
import { isOneOf } from "@/renderer/lib/one-of";
import { readPref, writePref } from "./view-prefs";

const PREVIEW_QUALITIES = ["fast", "exact"] as const;
export type PreviewQuality = (typeof PREVIEW_QUALITIES)[number];
export const isPreviewQuality = (v: unknown): v is PreviewQuality => isOneOf(PREVIEW_QUALITIES, v);

export const storedQuality = (): PreviewQuality =>
  readPref("previewQuality", "fast", isPreviewQuality);

const PREVIEW_SHAPES = ["project", ...VARIANTS] as const;
export type PreviewShape = (typeof PREVIEW_SHAPES)[number];
export const isPreviewShape = (v: unknown): v is PreviewShape => isOneOf(PREVIEW_SHAPES, v);

export const storedShape = (): PreviewShape => readPref("previewShape", "project", isPreviewShape);

export const singleSelected = (ids: string[]): string | null => (ids.length === 1 ? ids[0] : null);

export const storedMinimap = (): boolean =>
  readPref("minimap", true, (v) => typeof v === "boolean");

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
  minimap: boolean;
  setMinimap: (minimap: boolean) => void;
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

export const REPLAY_TABS = ["replay", "trace"] as const;
export type ReplayTab = (typeof REPLAY_TABS)[number];

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
  minimap: storedMinimap(),
  setMinimap: (minimap) => {
    set({ minimap });
    writePref("minimap", minimap);
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
