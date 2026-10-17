import { create } from "zustand";
import type { MixPlan } from "@/shared/mix/plan";
import type { AnalysisResult, Filmstrip, MixResult, Waveform } from "@/shared/mix/result";
import { debounce } from "@/shared/timing";
import { useAudioPreview } from "./audio-preview";

type Entry<T> = T | "loading" | "error";

type MediaState = {
  projectDir: string | null;
  waveforms: Record<string, Entry<Waveform>>;
  filmstrips: Record<string, Entry<Filmstrip>>;
  mixes: Partial<Record<string, MixResult>>;
  latest: Record<string, string>;
  mixing: string | null;
  mixError: string | null;
  failedKey: string | null;
  unavailable: boolean;
  reset: (projectDir: string | null) => void;
  waveform: (src: string) => void;
  filmstrip: (src: string) => void;
  requestMix: (source: string, plan: MixPlan) => void;
};

const MAX_MIXES = 8;
const MIX_DEBOUNCE_MS = 250;

const engineMissing = (r: AnalysisResult) => r.kind === "error" && !!r.engineMissing;

const emptyMedia = (projectDir: string | null) => ({
  projectDir,
  waveforms: {},
  filmstrips: {},
  mixes: {},
  latest: {},
  mixing: null,
  mixError: null,
  failedKey: null,
  unavailable: false,
});

function keepRecent(
  mixes: MediaState["mixes"],
  latest: MediaState["latest"],
  key: string,
  mix: MixResult,
  source: string,
) {
  const kept = new Set(
    Object.keys(mixes)
      .filter((k) => k !== key)
      .slice(1 - MAX_MIXES),
  );
  return {
    mixes: { ...Object.fromEntries([...kept].map((k) => [k, mixes[k]])), [key]: mix },
    latest: {
      ...Object.fromEntries(Object.entries(latest).filter(([, k]) => k === key || kept.has(k))),
      [source]: key,
    },
  };
}

export const useMedia = create<MediaState>()((set, get) => {
  const mixDone = (projectDir: string, source: string, plan: MixPlan, r: AnalysisResult) => {
    if (get().projectDir !== projectDir || r.kind === "skipped") return;
    if (r.kind === "error") {
      set((s) => ({
        mixError: r.error,
        failedKey: plan.key,
        unavailable: engineMissing(r),
        mixing: s.mixing === plan.key ? null : s.mixing,
      }));
      useAudioPreview.getState().clear();
      return;
    }
    if (r.kind !== "mix") return;
    set((s) => ({
      ...keepRecent(s.mixes, s.latest, plan.key, r.mix, source),
      mixing: s.mixing === plan.key ? null : s.mixing,
      mixError: null,
      failedKey: null,
    }));
    useAudioPreview.getState().clear();
  };

  const startMix = debounce((projectDir: string, source: string, plan: MixPlan) => {
    void window.demoflow
      .analyze({ kind: "mix", projectDir, plan })
      .then((r) => mixDone(projectDir, source, plan, r));
  }, MIX_DEBOUNCE_MS);

  const loadAnalysis = <T>(
    kind: "waveform" | "filmstrip",
    src: string,
    entries: (s: MediaState) => Record<string, Entry<T>>,
    store: (next: Record<string, Entry<T>>) => Partial<MediaState>,
    pick: (r: AnalysisResult) => T | undefined,
  ) => {
    const { projectDir, unavailable } = get();
    if (!projectDir || entries(get())[src] || unavailable) return;
    set((s) => store({ ...entries(s), [src]: "loading" }));
    void window.demoflow.analyze({ kind, projectDir, src }).then((r) => {
      if (get().projectDir !== projectDir) return;
      if (engineMissing(r)) set({ unavailable: true });
      set((s) => store({ ...entries(s), [src]: pick(r) ?? "error" }));
    });
  };

  return {
    ...emptyMedia(null),

    reset: (projectDir) => set(emptyMedia(projectDir)),

    waveform: (src) =>
      loadAnalysis(
        "waveform",
        src,
        (s) => s.waveforms,
        (waveforms) => ({ waveforms }),
        (r) => (r.kind === "waveform" ? r.waveform : undefined),
      ),

    filmstrip: (src) =>
      loadAnalysis(
        "filmstrip",
        src,
        (s) => s.filmstrips,
        (filmstrips) => ({ filmstrips }),
        (r) => (r.kind === "filmstrip" ? r.filmstrip : undefined),
      ),

    requestMix: (source, plan) => {
      const { projectDir, mixes, unavailable } = get();
      if (!projectDir || unavailable) return;
      if (mixes[plan.key]) {
        if (get().latest[source] !== plan.key) {
          set((s) => ({ latest: { ...s.latest, [source]: plan.key } }));
          useAudioPreview.getState().clear();
        }
        return;
      }
      set({ mixing: plan.key });
      startMix(projectDir, source, plan);
    },
  };
});
