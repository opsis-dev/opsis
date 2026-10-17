import { channelOf, soundLength } from "../audio";
import type { AudioItem, TimelineSpec } from "../schema/timeline";
import { soundNodeId, walkItems } from "./clip";

export type Span = readonly [number, number];

function mergeSpans(spans: Span[]): Span[] {
  const sorted = spans.filter(([a, b]) => b > a).toSorted((x, y) => x[0] - y[0]);
  const out: Array<[number, number]> = [];
  for (const [a, b] of sorted) {
    const last = out.at(-1);
    if (last && a <= last[1]) last[1] = Math.max(last[1], b);
    else out.push([a, b]);
  }
  return out;
}

function soundSpans(
  audio: AudioItem[],
  keys: ReadonlySet<string>,
  fps: number,
  total: number,
): Span[] {
  return audio
    .filter((a) => keys.has(channelOf(a)) && !a.keepMusic)
    .map((a) => [a.from, a.from + soundLength(a, fps, total)] as const);
}

function clipSpans(
  spec: Pick<TimelineSpec, "tracks" | "durationInFrames">,
  keys: ReadonlySet<string>,
): Span[] {
  const spans: Span[] = [];
  const film = { from: 0, to: spec.durationInFrames };
  walkItems(
    spec.tracks,
    (item, { from, to }) => {
      const id = soundNodeId(item);
      if (id && keys.has(id)) spans.push([from, to]);
    },
    film,
  );
  return spans;
}

export function duckingSpans(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
  keys: ReadonlySet<string>,
): Span[] {
  return mergeSpans([
    ...soundSpans(spec.audio, keys, spec.fps, spec.durationInFrames),
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
