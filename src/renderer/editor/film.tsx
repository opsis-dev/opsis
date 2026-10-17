import { createContext, useContext, useEffect, useMemo, type ReactNode } from "react";
import { applyMixLevels, resolveMix, withMasterStem } from "@/shared/mix/levels";
import { buildMixPlan, hasSound, type MixPlan } from "@/shared/mix/plan";
import type { MixResult } from "@/shared/mix/result";
import type { Mix } from "@/shared/mix/types";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { flattenSpec, type FlatTimeline } from "@/shared/timeline/flatten";
import { useMedia } from "../store/media";
import { useWorkflowStore } from "../store/workflow";
import { useCompiledContext } from "./use-compiled";

export type Film = {
  sourceNodeId: string | null;
  spec: TimelineSpec | null;
  flat: FlatTimeline | null;
  mix: Mix;
  plan: MixPlan | null;
  result: MixResult | null;
  stale: boolean;
  failed: boolean;
  mixing: boolean;
  audible: TimelineSpec | null;
};

const EMPTY: Film = {
  sourceNodeId: null,
  spec: null,
  flat: null,
  mix: resolveMix(undefined),
  plan: null,
  result: null,
  stale: false,
  failed: false,
  mixing: false,
  audible: null,
};

const FILM_SOURCE = "film";

function useFilmState(): Film {
  const { preview } = useCompiledContext();
  const storedMix = useWorkflowStore((s) => s.workflow?.mix);
  const mix = useMemo(() => resolveMix(storedMix), [storedMix]);
  const spec = preview?.spec ?? null;
  const source = preview?.sourceNodeId ?? FILM_SOURCE;
  const flat = useMemo(() => (spec ? flattenSpec(spec) : null), [spec]);
  const plan = useMemo(
    () => (spec && flat ? buildMixPlan(spec, mix, flat) : null),
    [spec, flat, mix],
  );
  const sounding = plan !== null && hasSound(plan) ? plan : null;
  const latest = useMedia((s) => s.latest[source]);
  const result = useMedia((s) => (latest ? (s.mixes[latest] ?? null) : null));
  const mixing = useMedia((s) => s.mixing !== null);
  const failedKey = useMedia((s) => s.failedKey);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  useEffect(() => {
    if (!sounding || !projectDir || result?.key === sounding.key) return;
    useMedia.getState().requestMix(source, sounding);
  }, [sounding, source, projectDir, result]);
  const audible = useMemo(() => {
    if (!spec) return null;
    if (result?.master && sounding) return withMasterStem(spec, result.master);
    return applyMixLevels(spec, mix);
  }, [spec, result, sounding, mix]);
  if (!preview) return EMPTY;
  return {
    sourceNodeId: preview.sourceNodeId,
    spec,
    flat,
    mix,
    plan,
    result,
    stale: !!sounding && result?.key !== sounding.key && failedKey !== sounding.key,
    failed: !!plan && failedKey === plan.key,
    mixing,
    audible,
  };
}

const FilmContext = createContext<Film>(EMPTY);

export function FilmProvider({ children }: { children: ReactNode }) {
  const film = useFilmState();
  return <FilmContext.Provider value={film}>{children}</FilmContext.Provider>;
}

export const useFilm = () => useContext(FilmContext);
