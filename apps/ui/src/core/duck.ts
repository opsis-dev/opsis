import type { AudioItem, FlatTimeline, TimelineSpec } from "@/generated/bindings";

export type Span = readonly [number, number];

const channelOf = (a: Pick<AudioItem, "channel" | "nodeId" | "id">): string =>
  a.channel ?? a.nodeId ?? a.id;

function soundLength(a: AudioItem, total: number): number {
  const rest = Math.max(0, total - a.from);
  if (a.role === "effect") {
    const natural = a.lengthFrames === undefined ? rest : a.lengthFrames / (a.rate ?? 1);
    return Math.min(natural, a.durationInFrames ?? natural);
  }
  return a.durationInFrames ?? rest;
}

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

function soundSpans(audio: AudioItem[], keys: ReadonlySet<string>, total: number): Span[] {
  return audio
    .filter((a) => keys.has(channelOf(a)) && !a.keepMusic)
    .map((a) => [a.from, a.from + soundLength(a, total)] as const);
}

const clipSpans = (flat: Pick<FlatTimeline, "audio">, keys: ReadonlySet<string>): Span[] =>
  flat.audio
    .filter((a) => a.role === "clip" && keys.has(a.channel))
    .map((a) => [a.from, a.to] as const);

export function duckingSpans(
  spec: Pick<TimelineSpec, "audio" | "durationInFrames">,
  keys: ReadonlySet<string>,
  flat: Pick<FlatTimeline, "audio">,
): Span[] {
  return mergeSpans([
    ...soundSpans(spec.audio, keys, spec.durationInFrames),
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
