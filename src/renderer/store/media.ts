import { create } from "zustand";
import type { MixPlan } from "@/shared/mix/plan";
import type {
  AnalysisKind,
  AnalysisRequest,
  AnalysisResult,
  AnalysisResultFor,
  Filmstrip,
  Waveform,
} from "@/shared/analysis";
import { errorMessage } from "@/shared/errors";
import { keepLast } from "@/shared/lists";
import type { MixResult } from "@/shared/mix/result";
import { debounce } from "@/shared/timing";
import { useAudioPreview } from "./audio-preview";

type Entry<T> = T | "loading" | "error";

type MediaOf = { waveform: Waveform; filmstrip: Filmstrip };
type MediaKind = keyof MediaOf;
type Analyses = { [K in MediaKind]: Record<string, Entry<MediaOf[K]>> };

type MediaState = {
  projectDir: string | null;
  analyses: Analyses;
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
  analyses: { waveform: {}, filmstrip: {} },
  mixes: {},
  latest: {},
  mixing: null,
  mixError: null,
  failedKey: null,
  unavailable: false,
});

function withNewestMix(
  mixes: MediaState["mixes"],
  key: string,
  mix: MixResult,
): MediaState["mixes"] {
  const older = keepLast(
    Object.keys(mixes).filter((k) => k !== key),
    MAX_MIXES - 1,
  );
  return { ...Object.fromEntries(older.map((k) => [k, mixes[k]])), [key]: mix };
}

const latestStillKept = (
  latest: MediaState["latest"],
  mixes: MediaState["mixes"],
): MediaState["latest"] =>
  Object.fromEntries(Object.entries(latest).filter(([, k]) => Object.hasOwn(mixes, k)));

function storeMix(s: MediaState, source: string, key: string, mix: MixResult) {
  const mixes = withNewestMix(s.mixes, key, mix);
  return { mixes, latest: { ...latestStillKept(s.latest, mixes), [source]: key } };
}

export const useMedia = create<MediaState>()((set, get) => ({
  ...emptyMedia(null),

  reset: (projectDir) => set(emptyMedia(projectDir)),

  waveform: (src) => loadAnalysis("waveform", src),

  filmstrip: (src) => loadAnalysis("filmstrip", src),

  requestMix: (source, plan) => {
    const { projectDir, mixes, unavailable } = get();
    if (!projectDir || unavailable) return;
    if (mixes[plan.key]) {
      if (get().latest[source] !== plan.key)
        setMixClearingPreview((s) => ({ latest: { ...s.latest, [source]: plan.key } }));
      return;
    }
    set({ mixing: plan.key });
    startLatestMix(projectDir, source, plan);
  },
}));

function setMixClearingPreview(update: (s: MediaState) => Partial<MediaState>): void {
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
    setMixClearingPreview((s) => ({
      mixError: r.error,
      failedKey: plan.key,
      unavailable: engineMissing(r),
      mixing: mixing(s),
    }));
  else
    setMixClearingPreview((s) => ({
      ...storeMix(s, source, plan.key, r.mix),
      mixing: mixing(s),
      mixError: null,
      failedKey: null,
    }));
}

function analyze<K extends AnalysisKind>(
  request: Extract<AnalysisRequest, { kind: K }>,
): Promise<AnalysisResultFor<K>> {
  return window.demoflow
    .analyze<K>(request)
    .catch((e: unknown): AnalysisResultFor<K> => ({ kind: "error", error: errorMessage(e) }));
}

const startLatestMix = debounce((projectDir: string, source: string, plan: MixPlan) => {
  void analyze<"mix">({ kind: "mix", projectDir, plan }).then((r) =>
    mixDone(projectDir, source, plan, r),
  );
}, MIX_DEBOUNCE_MS);

const MEDIA_VALUE: { [K in MediaKind]: (r: AnalysisResult) => MediaOf[K] | undefined } = {
  waveform: (r) => (r.kind === "waveform" ? r.waveform : undefined),
  filmstrip: (r) => (r.kind === "filmstrip" ? r.filmstrip : undefined),
};

function loadAnalysis<K extends MediaKind>(kind: K, src: string): void {
  const { projectDir, unavailable, analyses } = useMedia.getState();
  if (!projectDir || analyses[kind][src] || unavailable) return;
  const put = (entry: Entry<MediaOf[K]>) =>
    useMedia.setState((s) => ({
      analyses: { ...s.analyses, [kind]: { ...s.analyses[kind], [src]: entry } },
    }));
  put("loading");
  const request: Extract<AnalysisRequest, { kind: MediaKind }> = { kind, projectDir, src };
  void analyze(request).then((r) => {
    if (useMedia.getState().projectDir !== projectDir) return;
    if (engineMissing(r)) useMedia.setState({ unavailable: true });
    put(MEDIA_VALUE[kind](r) ?? "error");
  });
}

export const loaded = <T extends object>(entry: Entry<T> | undefined): T | null =>
  typeof entry === "object" ? entry : null;

export const loadedWaveform =
  (src: string) =>
  (s: MediaState): Waveform | null =>
    loaded(s.analyses.waveform[src]);

export const loadedFilmstrip =
  (src: string | null) =>
  (s: MediaState): Filmstrip | null =>
    src ? loaded(s.analyses.filmstrip[src]) : null;
