import { create } from "zustand";
import type { MixPlan } from "@/shared/mix/plan";
import type { AnalysisResult, AnalysisResultFor, Filmstrip, Waveform } from "@/shared/analysis";
import { keepLast } from "@/shared/lists";
import type { MixResult } from "@/shared/mix/result";
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
    keepLast(
      Object.keys(mixes).filter((k) => k !== key),
      MAX_MIXES - 1,
    ),
  );
  return {
    mixes: { ...Object.fromEntries([...kept].map((k) => [k, mixes[k]])), [key]: mix },
    latest: {
      ...Object.fromEntries(Object.entries(latest).filter(([, k]) => k === key || kept.has(k))),
      [source]: key,
    },
  };
}

export const useMedia = create<MediaState>()((set, get) => ({
  ...emptyMedia(null),

  reset: (projectDir) => set(emptyMedia(projectDir)),

  waveform: (src) => loadAnalysis(WAVEFORMS, src),

  filmstrip: (src) => loadAnalysis(FILMSTRIPS, src),

  requestMix: (source, plan) => {
    const { projectDir, mixes, unavailable } = get();
    if (!projectDir || unavailable) return;
    if (mixes[plan.key]) {
      if (get().latest[source] !== plan.key)
        setMix((s) => ({ latest: { ...s.latest, [source]: plan.key } }));
      return;
    }
    set({ mixing: plan.key });
    startMix(projectDir, source, plan);
  },
}));

function setMix(update: (s: MediaState) => Partial<MediaState>): void {
  useMedia.setState(update);
  useAudioPreview.getState().clear();
}

function mixDone(
  projectDir: string,
  source: string,
  plan: MixPlan,
  r: AnalysisResultFor<"mix">,
): void {
  if (useMedia.getState().projectDir !== projectDir || r.kind === "skipped") return;
  const mixing = (s: MediaState) => (s.mixing === plan.key ? null : s.mixing);
  if (r.kind === "error")
    setMix((s) => ({
      mixError: r.error,
      failedKey: plan.key,
      unavailable: engineMissing(r),
      mixing: mixing(s),
    }));
  else
    setMix((s) => ({
      ...keepRecent(s.mixes, s.latest, plan.key, r.mix, source),
      mixing: mixing(s),
      mixError: null,
      failedKey: null,
    }));
}

const startMix = debounce((projectDir: string, source: string, plan: MixPlan) => {
  void window.demoflow
    .analyze({ kind: "mix", projectDir, plan })
    .then((r) => mixDone(projectDir, source, plan, r));
}, MIX_DEBOUNCE_MS);

type Analysis<T> = {
  kind: "waveform" | "filmstrip";
  entries: (s: MediaState) => Record<string, Entry<T>>;
  store: (entries: Record<string, Entry<T>>) => Partial<MediaState>;
  value: (r: AnalysisResult) => T | undefined;
};

const WAVEFORMS: Analysis<Waveform> = {
  kind: "waveform",
  entries: (s) => s.waveforms,
  store: (waveforms) => ({ waveforms }),
  value: (r) => (r.kind === "waveform" ? r.waveform : undefined),
};

const FILMSTRIPS: Analysis<Filmstrip> = {
  kind: "filmstrip",
  entries: (s) => s.filmstrips,
  store: (filmstrips) => ({ filmstrips }),
  value: (r) => (r.kind === "filmstrip" ? r.filmstrip : undefined),
};

function loadAnalysis<T>(analysis: Analysis<T>, src: string): void {
  const { projectDir, unavailable } = useMedia.getState();
  if (!projectDir || analysis.entries(useMedia.getState())[src] || unavailable) return;
  const put = (entry: Entry<T>) =>
    useMedia.setState((s) => analysis.store({ ...analysis.entries(s), [src]: entry }));
  put("loading");
  void window.demoflow.analyze({ kind: analysis.kind, projectDir, src }).then((r) => {
    if (useMedia.getState().projectDir !== projectDir) return;
    if (engineMissing(r)) useMedia.setState({ unavailable: true });
    put(analysis.value(r) ?? "error");
  });
}

export const loaded = <T extends object>(entry: Entry<T> | undefined): T | null =>
  typeof entry === "object" ? entry : null;

export const loadedWaveform =
  (src: string) =>
  (s: MediaState): Waveform | null =>
    loaded(s.waveforms[src]);

export const loadedFilmstrip =
  (src: string | null) =>
  (s: MediaState): Filmstrip | null =>
    src ? loaded(s.filmstrips[src]) : null;
