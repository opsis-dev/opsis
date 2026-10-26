import { msToSeconds } from "@opsis/video/common/time";
import { useMemo } from "react";
import type { Waveform, AudioBar, FlatTimeline } from "@/generated/bindings";
import { loaded, useMedia } from "../../store/media";

const MAX_LOOPS = 200;

function barBeats(a: AudioBar, wave: Waveform, fps: number): number[] {
  const out: number[] = [];
  const len = msToSeconds(wave.durationMs);
  const offset = a.trimBefore / fps;
  for (let cycle = 0; cycle < (a.loop ? MAX_LOOPS : 1); cycle++) {
    const before = out.length;
    for (const b of wave.beats) {
      const f = a.from + (b + cycle * len - offset) * fps;
      if (f < a.from) continue;
      if (f >= a.to) break;
      out.push(f);
    }
    if (out.length === before && cycle > 0) break;
  }
  return out;
}

export function useBeatFrames(flat: FlatTimeline): number[] {
  const waveforms = useMedia((s) => s.analyses.waveform);
  return useMemo(
    () =>
      flat.audio.flatMap((a) => {
        const wave = loaded(waveforms[a.src]);
        if (a.role !== "music" || !wave?.beats.length) return [];
        return barBeats(a, wave, flat.fps);
      }),
    [flat, waveforms],
  );
}
