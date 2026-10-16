import { create } from "zustand";
import type { StripSettings } from "@/shared/mix/types";

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
