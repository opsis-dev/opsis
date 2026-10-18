import { channelOf, soundLength } from "../audio";
import type { AudioItem, TimelineSpec } from "../schema/timeline";
import { flattenSpec, type FlatTimeline } from "../timeline/flatten";

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

const clipSpans = (flat: FlatTimeline, keys: ReadonlySet<string>): Span[] =>
  flat.audio
    .filter((a) => a.role === "clip" && keys.has(a.channel))
    .map((a) => [a.from, a.to] as const);

export function duckingSpans(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
  keys: ReadonlySet<string>,
  flat: FlatTimeline = flattenSpec(spec),
): Span[] {
  return mergeSpans([
    ...soundSpans(spec.audio, keys, spec.fps, spec.durationInFrames),
    ...clipSpans(flat, keys),
  ]);
}

function spanGain(frame: number, [start, end]: Span, level: number, fadeFrames: number): number {
  if (frame >= start && frame < end) return level;
  const away = frame < start ? start - frame : frame - end;
  if (fadeFrames <= 0 || away >= fadeFrames) return 1;
  return level + (away / fadeFrames) * (1 - level);
}

export function duckGain(frame: number, spans: Span[], level: number, fadeFrames: number): number {
  let gain = 1;
  for (const span of spans) gain = Math.min(gain, spanGain(frame, span, level, fadeFrames));
  return gain;
}
