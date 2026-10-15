import type { AudioItem, TimelineItem, TimelineSpec, TimelineTrack } from "../schema/timeline";
import type { GroupProps } from "./clip";

export type DuckUnder = "all" | "effects" | "clips";
export type DuckSettings = { level: number; fadeFrames: number; under: DuckUnder };
export type Span = readonly [number, number];

export function mergeSpans(spans: Span[]): Span[] {
  const sorted = spans.filter(([a, b]) => b > a).toSorted((x, y) => x[0] - y[0]);
  const out: Array<[number, number]> = [];
  for (const [a, b] of sorted) {
    const last = out.at(-1);
    if (last && a <= last[1]) last[1] = Math.max(last[1], b);
    else out.push([a, b]);
  }
  return out;
}

export function effectSpans(audio: AudioItem[], fps: number): Span[] {
  return audio
    .filter((a) => a.role === "effect" && !a.keepMusic)
    .map((a) => {
      const natural = a.lengthFrames ?? fps;
      const length = Math.min(natural, a.durationInFrames ?? natural);
      return [a.from, a.from + length] as const;
    });
}

function clipSpansIn(tracks: TimelineTrack[], offset: number, end: number): Span[] {
  return tracks.flatMap((track) =>
    track.items.flatMap((item: TimelineItem): Span[] => {
      const from = offset + item.from;
      const to = Math.min(end, from + item.durationInFrames);
      if (item.component === "core/video" && item.props.muted === false) return [[from, to]];
      if (item.component === "core/group") {
        const inner = (item.props as unknown as GroupProps).tracks ?? [];
        return clipSpansIn(inner, from, to);
      }
      return [];
    }),
  );
}

export function clipSpans(spec: Pick<TimelineSpec, "tracks" | "durationInFrames">): Span[] {
  return clipSpansIn(spec.tracks, 0, spec.durationInFrames);
}

export function duckingSpans(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
  under: DuckUnder,
): Span[] {
  const effects = under === "clips" ? [] : effectSpans(spec.audio ?? [], spec.fps);
  const clips = under === "effects" ? [] : clipSpans(spec);
  return mergeSpans([...effects, ...clips]);
}

export function duckGain(frame: number, spans: Span[], level: number, fadeFrames: number): number {
  let gain = 1;
  for (const [start, end] of spans) {
    let g = 1;
    if (frame >= start && frame < end) g = level;
    else if (fadeFrames > 0 && frame >= start - fadeFrames && frame < start)
      g = 1 - ((frame - (start - fadeFrames)) / fadeFrames) * (1 - level);
    else if (fadeFrames > 0 && frame >= end && frame < end + fadeFrames)
      g = level + ((frame - end) / fadeFrames) * (1 - level);
    gain = Math.min(gain, g);
  }
  return gain;
}
