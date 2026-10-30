import { lookupKey } from "@opsis/video/common/hash";
import { createContext, useContext, useEffect, useMemo, type ReactNode } from "react";
import type {
  LocalFrames,
  FlatTimeline,
  Mix,
  MixPlan,
  MixResult,
  TimelineSpec,
} from "@/generated/bindings";
import { mixFailure, useMedia } from "../store/media";
import { type FileVersions, useProjectFiles, versionsOf } from "../store/project-files";
import { useWorkflowStore } from "../store/workflow";
import { useCompiledContext } from "./use-compiled";

export type Film = {
  sourceNodeId: string | null;
  spec: TimelineSpec | null;
  flat: FlatTimeline | null;
  mix: Mix | null;
  solo: boolean;
  plan: MixPlan | null;
  result: MixResult | null;
  stale: boolean;
  failed: boolean;
  mixing: boolean;
  levelled: TimelineSpec | null;
  stem: string | null;
  localFrames: Partial<Record<string, LocalFrames>>;
};

const EMPTY: Film = {
  sourceNodeId: null,
  spec: null,
  flat: null,
  mix: null,
  solo: false,
  plan: null,
  result: null,
  stale: false,
  failed: false,
  mixing: false,
  levelled: null,
  stem: null,
  localFrames: {},
};

const FILM_SOURCE = "film";

function hasSound(plan: MixPlan): boolean {
  return plan.channels.some((c) => c.sounds.length > 0);
}

function keyedByFiles(plan: MixPlan, versions: FileVersions): MixPlan {
  const sounds = plan.channels.flatMap((c) => c.sounds.map((s) => s.src));
  const files = versionsOf(sounds, versions);
  return Object.keys(files).length ? { ...plan, key: lookupKey([plan.key, files]) } : plan;
}

type MixState = { result: MixResult | null; mixing: boolean; failedKey: string | null };

function useMixResult(sounding: MixPlan | null, source: string): MixState {
  const latest = useMedia((s) => s.mix.latest[source]);
  const result = useMedia((s) => (latest ? (s.mix.results[latest] ?? null) : null));
  const mixing = useMedia((s) => s.mix.status.kind === "pending");
  const failedKey = useMedia((s) => mixFailure(s)?.key ?? null);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const current = !!sounding && result?.key === sounding.key;
  useEffect(() => {
    if (!sounding || !projectDir || current) return;
    useMedia.getState().requestMix(source, sounding);
  }, [sounding, source, projectDir, current]);
  return { result, mixing, failedKey };
}

function useFilmState(): Film {
  const { film, flat, plan: planned, levelled, localFrames, mix, solo } = useCompiledContext();
  const spec = film?.spec ?? null;
  const versions = useProjectFiles((s) => s.versions);
  const plan = useMemo(
    () => (planned ? keyedByFiles(planned, versions) : null),
    [planned, versions],
  );
  const sounding = plan !== null && hasSound(plan) ? plan : null;
  const { result, mixing, failedKey } = useMixResult(sounding, film?.sourceNodeId ?? FILM_SOURCE);
  if (!film) return EMPTY;
  const current = !!sounding && result?.key === sounding.key;
  return {
    sourceNodeId: film.sourceNodeId,
    spec,
    flat,
    mix,
    solo,
    plan,
    result,
    stale: !!sounding && !current && failedKey !== sounding.key,
    failed: !!plan && failedKey === plan.key,
    mixing,
    levelled,
    stem: result?.master && sounding ? result.master.src : null,
    localFrames,
  };
}

const FilmContext = createContext<Film>(EMPTY);

export function FilmProvider({ children }: { children: ReactNode }) {
  const film = useFilmState();
  return <FilmContext.Provider value={film}>{children}</FilmContext.Provider>;
}

export function useFilm() {
  return useContext(FilmContext);
}
