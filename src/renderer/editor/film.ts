import { createContext, useContext, useEffect, useMemo } from "react";
import { applyMixLevels, withMasterStem } from "@/shared/mix/levels";
import { buildMixPlan, hasSound, type MixPlan } from "@/shared/mix/plan";
import type { MixResult } from "@/shared/mix/result";
import { resolveMix } from "@/shared/mix/levels";
import type { Mix } from "@/shared/mix/types";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { flattenSpec, type FlatTimeline } from "@/shared/timeline/flatten";
import { useMedia } from "../store/media";
import { useWorkflowStore } from "../store/workflow";
import { useCompiledContext } from "./use-compiled";

export type Film = {
  source: string;
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
  source: "film",
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

export function useFilmState(): Film {
  const { preview } = useCompiledContext();
  const storedMix = useWorkflowStore((s) => s.workflow?.mix);
  const mix = useMemo(() => resolveMix(storedMix), [storedMix]);
  const spec = preview?.spec ?? null;
  const source = preview?.sourceNodeId ?? "film";
  const flat = useMemo(() => (spec ? flattenSpec(spec) : null), [spec]);
  const plan = useMemo(
    () => (spec && flat ? buildMixPlan(spec, mix, flat) : null),
    [spec, flat, mix],
  );
  const latest = useMedia((s) => s.latest[source]);
  const result = useMedia((s) => (latest ? (s.mixes[latest] ?? null) : null));
  const mixing = useMedia((s) => s.mixing !== null);
  const failedKey = useMedia((s) => s.failedKey);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  useEffect(() => {
    if (plan && hasSound(plan) && projectDir) useMedia.getState().requestMix(source, plan);
  }, [plan, source, projectDir]);
  const audible = useMemo(() => {
    if (!spec) return null;
    if (result?.master && plan && hasSound(plan)) return withMasterStem(spec, result.master);
    return applyMixLevels(spec, mix);
  }, [spec, result, plan, mix]);
  if (!preview) return EMPTY;
  return {
    source,
    sourceNodeId: preview.sourceNodeId,
    spec,
    flat,
    mix,
    plan,
    result,
    stale: !!plan && hasSound(plan) && result?.key !== plan.key && failedKey !== plan.key,
    failed: !!plan && failedKey === plan.key,
    mixing,
    audible,
  };
}

export const FilmContext = createContext<Film>(EMPTY);
export const useFilm = () => useContext(FilmContext);
