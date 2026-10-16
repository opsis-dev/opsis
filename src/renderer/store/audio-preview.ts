import { create } from "zustand";

export type PreviewFilter = { type: BiquadFilterType; freq: number; gain: number; q: number };

type AudioPreviewState = {
  gainDb: number;
  pan: number;
  filters: PreviewFilter[];
  set: (patch: Partial<Pick<AudioPreviewState, "gainDb" | "pan" | "filters">>) => void;
  clear: () => void;
};

export const useAudioPreview = create<AudioPreviewState>()((set) => ({
  gainDb: 0,
  pan: 0,
  filters: [],
  set: (patch) => set(patch),
  clear: () => set({ gainDb: 0, pan: 0, filters: [] }),
}));
