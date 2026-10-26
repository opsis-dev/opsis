import { useMemo } from "react";
import { duckGain, duckingSpans } from "@/core/duck";
import type { FlatTimeline, TimelineSpec } from "@/generated/bindings";
import type { Gain } from "./bar-canvas";

const NO_GAIN: Gain = () => 1;

type DuckedFilm = { levelled: TimelineSpec | null; flat: FlatTimeline | null };

export function useDuckGains({ levelled, flat }: DuckedFilm): (id: string) => Gain {
  const gains = useMemo(() => {
    const out = new Map<string, Gain>();
    if (!levelled || !flat) return out;
    for (const a of levelled.audio) {
      if (!a.duck) continue;
      const spans = duckingSpans(levelled, new Set(a.duck.keys), flat);
      const { level, fadeFrames } = a.duck;
      out.set(a.id, (abs) => duckGain(abs, spans, level, fadeFrames));
    }
    return out;
  }, [levelled, flat]);
  return (id) => gains.get(id) ?? NO_GAIN;
}
