import { useMemo } from "react";
import { duckGain, duckingSpans } from "@/shared/mix/duck";
import { applyMixLevels } from "@/shared/mix/levels";
import type { Mix } from "@/shared/mix/types";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { flattenSpec } from "@/shared/timeline/flatten";
import type { Gain } from "./bar-canvas";

const NO_GAIN: Gain = () => 1;

export function useDuckGains(spec: TimelineSpec | null, mix: Mix): (id: string) => Gain {
  const gains = useMemo(() => {
    const out = new Map<string, Gain>();
    if (!spec) return out;
    const live = applyMixLevels(spec, mix);
    if (!live.audio.some((a) => a.duck)) return out;
    const flat = flattenSpec(live);
    for (const a of live.audio) {
      if (!a.duck) continue;
      const spans = duckingSpans(live, new Set(a.duck.keys), flat);
      const { level, fadeFrames } = a.duck;
      out.set(a.id, (abs) => duckGain(abs, spans, level, fadeFrames));
    }
    return out;
  }, [spec, mix]);
  return (id) => gains.get(id) ?? NO_GAIN;
}
