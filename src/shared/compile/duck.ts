import type { AudioItem, TimelineItem, TimelineSpec, TimelineTrack } from "../schema/timeline";
import type { GroupProps } from "./clip";

export type Span = readonly [number, number];

export const channelOf = (a: Pick<AudioItem, "channel" | "nodeId" | "id">): string =>
  a.channel ?? a.nodeId ?? a.id;

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

export function soundLength(a: AudioItem, fps: number, total: number): number {
  if (a.role === "effect") {
    const natural = a.lengthFrames ?? fps;
    return Math.min(natural, a.durationInFrames ?? natural);
  }
  return a.durationInFrames ?? Math.max(0, total - a.from);
}

export function soundSpans(
  audio: AudioItem[],
  keys: ReadonlySet<string>,
  fps: number,
  total: number,
): Span[] {
  return audio
    .filter((a) => keys.has(channelOf(a)) && !a.keepMusic)
    .map((a) => [a.from, a.from + soundLength(a, fps, total)] as const);
}

function clipSpansIn(
  tracks: TimelineTrack[],
  offset: number,
  end: number,
  keys: ReadonlySet<string> | null,
): Span[] {
  return tracks.flatMap((track) =>
    track.items.flatMap((item: TimelineItem): Span[] => {
      const from = offset + item.from;
      const to = Math.min(end, from + item.durationInFrames);
      if (
        item.component === "core/video" &&
        item.props.muted === false &&
        (!keys || (item.nodeId !== undefined && keys.has(item.nodeId)))
      )
        return [[from, to]];
      if (item.component === "core/group") {
        const inner = (item.props as unknown as GroupProps).tracks ?? [];
        return clipSpansIn(inner, from, to, keys);
      }
      return [];
    }),
  );
}

export function clipSpans(
  spec: Pick<TimelineSpec, "tracks" | "durationInFrames">,
  keys: ReadonlySet<string> | null = null,
): Span[] {
  return clipSpansIn(spec.tracks, 0, spec.durationInFrames, keys);
}

export function duckingSpans(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
  keys: ReadonlySet<string>,
): Span[] {
  return mergeSpans([
    ...soundSpans(spec.audio ?? [], keys, spec.fps, spec.durationInFrames),
    ...clipSpans(spec, keys),
  ]);
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
