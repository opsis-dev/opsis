import { create } from "zustand";
import type { MixPlan } from "@/shared/mix/plan";
import type { Filmstrip, MixResult, Waveform } from "@/shared/mix/result";
import { isEngineMissing } from "@/shared/engine-message";
import { debounce } from "@/shared/timing";

type Entry<T> = T | "loading" | "error";

type MediaState = {
  projectDir: string | null;
  waveforms: Record<string, Entry<Waveform>>;
  filmstrips: Record<string, Entry<Filmstrip>>;
  mixes: Record<string, MixResult>;
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
export const useMedia = create<MediaState>()((set, get) => {
  const startMix = debounce((projectDir: string, source: string, plan: MixPlan) => {
    void window.demoflow.analyze({ kind: "mix", projectDir, plan }).then((r) => {
      if (get().projectDir !== projectDir) return;
      if (r.kind === "skipped") return;
      if (r.kind === "error") {
        set({
          mixError: r.error,
          failedKey: plan.key,
          unavailable: isEngineMissing(r.error),
          mixing: null,
        });
        return;
      }
      if (r.kind !== "mix") return;
      set((s) => {
        const keys = Object.keys(s.mixes)
          .filter((k) => k !== plan.key)
          .slice(-(MAX_MIXES - 1));
        return {
          mixes: { ...Object.fromEntries(keys.map((k) => [k, s.mixes[k]])), [plan.key]: r.mix },
          latest: { ...s.latest, [source]: plan.key },
          mixing: s.mixing === plan.key ? null : s.mixing,
          mixError: null,
          failedKey: null,
        };
      });
    });
  }, 250);

  return {
    projectDir: null,
    waveforms: {},
    filmstrips: {},
    mixes: {},
    latest: {},
    mixing: null,
    mixError: null,
    failedKey: null,
    unavailable: false,

    reset: (projectDir) =>
      set({
        projectDir,
        waveforms: {},
        filmstrips: {},
        mixes: {},
        latest: {},
        mixing: null,
        mixError: null,
        failedKey: null,
        unavailable: false,
      }),

    waveform: (src) => {
      const { projectDir, waveforms, unavailable } = get();
      if (!projectDir || waveforms[src] || unavailable) return;
      set((s) => ({ waveforms: { ...s.waveforms, [src]: "loading" } }));
      void window.demoflow.analyze({ kind: "waveform", projectDir, src }).then((r) => {
        if (get().projectDir !== projectDir) return;
        if (r.kind === "error" && isEngineMissing(r.error)) set({ unavailable: true });
        set((s) => ({
          waveforms: { ...s.waveforms, [src]: r.kind === "waveform" ? r.waveform : "error" },
        }));
      });
    },

    filmstrip: (src) => {
      const { projectDir, filmstrips, unavailable } = get();
      if (!projectDir || filmstrips[src] || unavailable) return;
      set((s) => ({ filmstrips: { ...s.filmstrips, [src]: "loading" } }));
      void window.demoflow.analyze({ kind: "filmstrip", projectDir, src }).then((r) => {
        if (get().projectDir !== projectDir) return;
        if (r.kind === "error" && isEngineMissing(r.error)) set({ unavailable: true });
        set((s) => ({
          filmstrips: { ...s.filmstrips, [src]: r.kind === "filmstrip" ? r.filmstrip : "error" },
        }));
      });
    },

    requestMix: (source, plan) => {
      const { projectDir, mixes, unavailable } = get();
      if (!projectDir || unavailable) return;
      if (mixes[plan.key]) {
        if (get().latest[source] !== plan.key)
          set((s) => ({ latest: { ...s.latest, [source]: plan.key } }));
        return;
      }
      set({ mixing: plan.key });
      startMix(projectDir, source, plan);
    },
  };
});
