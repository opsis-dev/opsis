import { create } from "zustand";
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
import type { MixPlan } from "@/shared/mix/plan";
import type { MixResult } from "@/shared/mix/result";
import { debounce } from "@/shared/timing";
import { useAudioPreview } from "./audio-preview";

type Entry<T> = T | "loading" | "error";

type MediaOf = { waveform: Waveform; filmstrip: Filmstrip };
type MediaKind = keyof MediaOf;
type Analyses = { [K in MediaKind]: Record<string, Entry<MediaOf[K]>> };

type MixState = {
  results: Partial<Record<string, MixResult>>;
  latest: Record<string, string>;
  pending: string | null;
  error: string | null;
  failedKey: string | null;
};

type MediaState = {
  projectDir: string | null;
  analyses: Analyses;
  mix: MixState;
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
  mix: { results: {}, latest: {}, pending: null, error: null, failedKey: null },
  unavailable: false,
});

function keepNewestResult(
  results: MixState["results"],
  key: string,
  result: MixResult,
): MixState["results"] {
  const older = keepLast(
    Object.keys(results).filter((k) => k !== key),
    MAX_MIXES - 1,
  );
  return { ...Object.fromEntries(older.map((k) => [k, results[k]])), [key]: result };
}

const pruneLatest = (
  latest: MixState["latest"],
  results: MixState["results"],
): MixState["latest"] =>
  Object.fromEntries(Object.entries(latest).filter(([, k]) => Object.hasOwn(results, k)));

function withResult(mix: MixState, source: string, key: string, result: MixResult): MixState {
  const results = keepNewestResult(mix.results, key, result);
  return { ...mix, results, latest: { ...pruneLatest(mix.latest, results), [source]: key } };
}

const pendingAfter = (mix: MixState, key: string): string | null =>
  mix.pending === key ? null : mix.pending;

function updateMixClearingPreview(
  update: (mix: MixState) => MixState,
  unavailable?: boolean,
): void {
  useMedia.setState((s) => ({ mix: update(s.mix), unavailable: unavailable ?? s.unavailable }));
  useAudioPreview.getState().clear();
}

function finishMix(
  projectDir: string,
  source: string,
  plan: MixPlan,
  r: AnalysisResultFor<"mix">,
): void {
  if (useMedia.getState().projectDir !== projectDir || r.kind === "skipped") return;
  if (r.kind === "error")
    updateMixClearingPreview(
      (mix) => ({
        ...mix,
        pending: pendingAfter(mix, plan.key),
        error: r.error,
        failedKey: plan.key,
      }),
      engineMissing(r),
    );
  else
    updateMixClearingPreview((mix) => ({
      ...withResult(mix, source, plan.key, r.mix),
      pending: pendingAfter(mix, plan.key),
      error: null,
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

const mixSoon = debounce((projectDir: string, source: string, plan: MixPlan) => {
  void analyze<"mix">({ kind: "mix", projectDir, plan }).then((r) =>
    finishMix(projectDir, source, plan, r),
  );
}, MIX_DEBOUNCE_MS);

export const useMedia = create<MediaState>()((set, get) => ({
  ...emptyMedia(null),

  reset: (projectDir) => set(emptyMedia(projectDir)),

  waveform: (src) => loadAnalysis("waveform", src),

  filmstrip: (src) => loadAnalysis("filmstrip", src),

  requestMix: (source, plan) => {
    const { projectDir, mix, unavailable } = get();
    if (!projectDir || unavailable) return;
    if (mix.results[plan.key]) {
      if (mix.latest[source] !== plan.key)
        updateMixClearingPreview((m) => ({ ...m, latest: { ...m.latest, [source]: plan.key } }));
      return;
    }
    set({ mix: { ...mix, pending: plan.key } });
    mixSoon(projectDir, source, plan);
  },
}));

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
  (src: string | null) =>
  (s: MediaState): Waveform | null =>
    src ? loaded(s.analyses.waveform[src]) : null;

export const loadedFilmstrip =
  (src: string | null) =>
  (s: MediaState): Filmstrip | null =>
    src ? loaded(s.analyses.filmstrip[src]) : null;
