import { applyMixLevels } from "@opsis/core/mix/levels";
import type { Mix } from "@opsis/core/mix/types";
import type { TimelineSpec } from "@opsis/core/schema/timeline";
import type { FlatTimeline } from "@opsis/core/timeline/flatten";
import { duckGain, duckingSpans } from "@opsis/video/spec";
import { useMemo } from "react";
import type { Gain } from "./bar-canvas";

const NO_GAIN: Gain = () => 1;

type DuckedFilm = { spec: TimelineSpec | null; flat: FlatTimeline | null; mix: Mix };

export function useDuckGains({ spec, flat, mix }: DuckedFilm): (id: string) => Gain {
  const gains = useMemo(() => {
    const out = new Map<string, Gain>();
    if (!spec || !flat) return out;
    const live = applyMixLevels(spec, mix, flat);
    for (const a of live.audio) {
      if (!a.duck) continue;
      const spans = duckingSpans(live, new Set(a.duck.keys), flat);
      const { level, fadeFrames } = a.duck;
      out.set(a.id, (abs) => duckGain(abs, spans, level, fadeFrames));
    }
    return out;
  }, [spec, flat, mix]);
  return (id) => gains.get(id) ?? NO_GAIN;
}
