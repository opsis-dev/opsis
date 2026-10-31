import { keepLast } from "@opsis/video/common/lists";
import { create } from "zustand";
import {
  commands,
  type AnalysisRequest,
  type AnalysisResult,
  type Filmstrip,
  type MixPlan,
  type MixResult,
  type Waveform,
} from "@/generated/bindings";
import { errorMessage } from "@/lib/errors";
import { debounce } from "@/lib/timing";
import { useAudioPreview } from "./audio-preview";

type Entry<T> = T | "loading" | "error";

type AnalysisKind = AnalysisRequest["kind"];

type AnalysisResultFor<K extends AnalysisKind> = Extract<
  AnalysisResult,
  { kind: K | "skipped" | "error" }
>;

type MediaOf = { waveform: Waveform; filmstrip: Filmstrip };
type MediaKind = keyof MediaOf;
type Analyses = { [K in MediaKind]: Record<string, Entry<MediaOf[K]>> };

type MixStatus =
  | { kind: "idle" }
  | { kind: "pending"; key: string }
  | { kind: "failed"; key: string; error: string };

type MixState = {
  results: Partial<Record<string, MixResult>>;
  latest: Record<string, string>;
  status: MixStatus;
};

type MediaState = {
  projectDir: string | null;
  analyses: Analyses;
  mix: MixState;
  reset: (projectDir: string | null) => void;
  waveform: (src: string) => void;
  filmstrip: (src: string) => void;
  requestMix: (source: string, plan: MixPlan) => void;
  reloadChanged: (refs: readonly string[]) => void;
};

const MEDIA_KINDS: readonly MediaKind[] = ["waveform", "filmstrip"];

const MAX_MIXES = 8;
const MIX_DEBOUNCE_MS = 250;

const IDLE: MixStatus = { kind: "idle" };

function emptyMedia(projectDir: string | null) {
  return {
    projectDir,
    analyses: { waveform: {}, filmstrip: {} },
    mix: { results: {}, latest: {}, status: IDLE },
  };
}

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

function pruneLatest(latest: MixState["latest"], results: MixState["results"]): MixState["latest"] {
  return Object.fromEntries(Object.entries(latest).filter(([, k]) => Object.hasOwn(results, k)));
}

function waitingForAnother(status: MixStatus, key: string): boolean {
  return status.kind === "pending" && status.key !== key;
}

function mixDone(mix: MixState, source: string, key: string, result: MixResult): MixState {
  const results = keepNewestResult(mix.results, key, result);
  return {
    results,
    latest: { ...pruneLatest(mix.latest, results), [source]: key },
    status: waitingForAnother(mix.status, key) ? mix.status : IDLE,
  };
}

function mixFailed(mix: MixState, key: string, error: string): MixState {
  return {
    ...mix,
    status: waitingForAnother(mix.status, key) ? mix.status : { kind: "failed", key, error },
  };
}

function updateMixClearingPreview(update: (mix: MixState) => MixState): void {
  useMedia.setState((s) => ({ mix: update(s.mix) }));
  useAudioPreview.getState().clear();
}

function finishMix(
  projectDir: string,
  source: string,
  plan: MixPlan,
  r: AnalysisResultFor<"mix">,
): void {
  if (useMedia.getState().projectDir !== projectDir || r.kind === "skipped") return;
  if (r.kind === "error") updateMixClearingPreview((mix) => mixFailed(mix, plan.key, r.error));
  else updateMixClearingPreview((mix) => mixDone(mix, source, plan.key, r.mix));
}

function analyze<K extends AnalysisKind>(
  request: Extract<AnalysisRequest, { kind: K }>,
): Promise<AnalysisResultFor<K>> {
  return (commands.analyze(request) as Promise<AnalysisResultFor<K>>).catch(
    (e: unknown): AnalysisResultFor<K> => ({ kind: "error", error: errorMessage(e) }),
  );
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
    const { projectDir, mix } = get();
    if (!projectDir) return;
    if (mix.results[plan.key]) {
      if (mix.latest[source] !== plan.key)
        updateMixClearingPreview((m) => ({ ...m, latest: { ...m.latest, [source]: plan.key } }));
      return;
    }
    set({ mix: { ...mix, status: { kind: "pending", key: plan.key } } });
    mixSoon(projectDir, source, plan);
  },

  reloadChanged: (refs) => {
    const changed = new Set(refs);
    function isChanged(src: string) {
      return changed.has(src);
    }
    for (const kind of MEDIA_KINDS) {
      for (const src of Object.keys(get().analyses[kind]).filter(isChanged))
        loadAnalysis(kind, src, true);
    }
  },
}));

const MEDIA_VALUE: { [K in MediaKind]: (r: AnalysisResult) => MediaOf[K] | undefined } = {
  waveform: (r) => (r.kind === "waveform" ? r.waveform : undefined),
  filmstrip: (r) => (r.kind === "filmstrip" ? r.filmstrip : undefined),
};

function loadAnalysis<K extends MediaKind>(kind: K, src: string, again = false): void {
  const { projectDir, analyses } = useMedia.getState();
  if (!projectDir || (analyses[kind][src] && !again)) return;
  function put(entry: Entry<MediaOf[K]>) {
    return useMedia.setState((s) => ({
      analyses: { ...s.analyses, [kind]: { ...s.analyses[kind], [src]: entry } },
    }));
  }
  if (!again) put("loading");
  const request: Extract<AnalysisRequest, { kind: MediaKind }> = { kind, projectDir, src };
  void analyze(request).then((r) => {
    if (useMedia.getState().projectDir !== projectDir) return;
    put(MEDIA_VALUE[kind](r) ?? "error");
  });
}

export function mixFailure(s: MediaState) {
  return s.mix.status.kind === "failed" ? s.mix.status : null;
}

export function loaded<T extends object>(entry: Entry<T> | undefined): T | null {
  return typeof entry === "object" ? entry : null;
}

export function loadedWaveform(src: string | null) {
  return (s: MediaState): Waveform | null => (src ? loaded(s.analyses.waveform[src]) : null);
}

export function loadedFilmstrip(src: string | null) {
  return (s: MediaState): Filmstrip | null => (src ? loaded(s.analyses.filmstrip[src]) : null);
}
