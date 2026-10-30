import { useMemo } from "react";
import type { TimelineSpec } from "@/generated/bindings";
import { duckGain } from "@/timeline/duck";
import type { Gain } from "./bar-canvas";

function noGain(): number {
  return 1;
}

type DuckedFilm = { levelled: TimelineSpec | null };

export function useDuckGains({ levelled }: DuckedFilm): (id: string) => Gain {
  const gains = useMemo(() => {
    const out = new Map<string, Gain>();
    if (!levelled) return out;
    for (const a of levelled.audio) {
      if (!a.duck) continue;
      const { level, fadeFrames, spans } = a.duck;
      out.set(a.id, (abs) => duckGain(abs, spans, level, fadeFrames));
    }
    return out;
  }, [levelled]);
  return (id) => gains.get(id) ?? noGain;
}
