import { create } from "zustand";

type PreviewFilter = { type: BiquadFilterType; freq: number; gain: number; q: number };

export type Contribution = { gainDb?: number; pan?: number; filters?: PreviewFilter[] };

export const MAX_PREVIEW_FILTERS = 6;

type AudioPreviewState = {
  contributions: Record<string, Contribution>;
  baselines: Record<string, number>;
  contribute: (id: string, contribution: Contribution) => void;
  rememberBaseline: (id: string, current: number) => number;
  clear: () => void;
};

export function previewTotals({ contributions }: Pick<AudioPreviewState, "contributions">) {
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
  contributions: {},
  baselines: {},
  contribute: (id, contribution) =>
    set((s) => ({ contributions: { ...s.contributions, [id]: contribution } })),
  rememberBaseline: (id, current) => {
    const known = get().baselines[id];
    if (known !== undefined) return known;
    set((s) => ({ baselines: { ...s.baselines, [id]: current } }));
    return current;
  },
  clear: () => set({ contributions: {}, baselines: {} }),
}));
