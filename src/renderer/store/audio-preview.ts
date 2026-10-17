import { create } from "zustand";

type PreviewFilter = { type: BiquadFilterType; freq: number; gain: number; q: number };

export type Contribution = { gainDb?: number; pan?: number; filters?: PreviewFilter[] };

export const MAX_PREVIEW_FILTERS = 6;

type AudioPreviewState = {
  gainDb: number;
  pan: number;
  filters: PreviewFilter[];
  contributions: Record<string, Contribution>;
  baselines: Record<string, number>;
  contribute: (id: string, contribution: Contribution) => void;
  baseline: (id: string, current: number) => number;
  clear: () => void;
};

function combine(contributions: Record<string, Contribution>) {
  let gainDb = 0;
  let pan = 0;
  const filters: PreviewFilter[] = [];
  for (const c of Object.values(contributions)) {
    gainDb += c.gainDb ?? 0;
    pan += c.pan ?? 0;
    filters.push(...(c.filters ?? []));
  }
  return { gainDb, pan, filters: filters.slice(0, MAX_PREVIEW_FILTERS) };
}

export const useAudioPreview = create<AudioPreviewState>()((set, get) => ({
  gainDb: 0,
  pan: 0,
  filters: [],
  contributions: {},
  baselines: {},
  contribute: (id, contribution) =>
    set((s) => {
      const contributions = { ...s.contributions, [id]: contribution };
      return { contributions, ...combine(contributions) };
    }),
  baseline: (id, current) => {
    const known = get().baselines[id];
    if (known !== undefined) return known;
    set((s) => ({ baselines: { ...s.baselines, [id]: current } }));
    return current;
  },
  clear: () => set({ gainDb: 0, pan: 0, filters: [], contributions: {}, baselines: {} }),
}));
