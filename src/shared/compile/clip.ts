import type {
  AudioItem,
  TimelineItem,
  TimelineSpec,
  TimelineTrack,
  TransitionRef,
} from "../schema/timeline";
import type { ClipValue, CompileValue, RecordingValue } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";

export type Crop = { x: number; y: number; w: number; h: number };

export type GroupProps = {
  tracks: TimelineTrack[];
  startFrom?: number;
  remap?: Array<[number, number]>;
  loopEvery?: number;
  crop?: Crop;
  wrapper?: { component: string; props: Record<string, unknown> };
  background?: string;
};

export const msToFrames = (ms: number, fps: number) => Math.max(1, Math.round((ms / 1000) * fps));
export const framesToMs = (frames: number, fps: number) => (frames / fps) * 1000;

export const isClip = (v: CompileValue | undefined): v is ClipValue => v?.type === "clip";

export function emptyClip(durationInFrames = 1): ClipValue {
  return { type: "clip", durationInFrames, tracks: [], audio: [] };
}

export function itemClip(
  nodeId: string,
  component: string,
  props: Record<string, unknown>,
  durationInFrames: number,
  extra: { recording?: RecordingValue; audio?: AudioItem[] } = {},
): ClipValue {
  const duration = Math.max(1, Math.round(durationInFrames));
  return {
    type: "clip",
    durationInFrames: duration,
    tracks: [
      {
        id: `${nodeId}:t`,
        items: [
          { id: `${nodeId}:i`, from: 0, durationInFrames: duration, component, props, nodeId },
        ],
      },
    ],
    audio: extra.audio ?? [],
    recording: extra.recording,
  };
}

export function wrapClip(
  clip: ClipValue,
  nodeId: string,
  props: Omit<GroupProps, "tracks"> = {},
  durationInFrames = clip.durationInFrames,
): ClipValue {
  const duration = Math.max(1, Math.round(durationInFrames));
  const group: TimelineItem = {
    id: `${nodeId}:g`,
    from: 0,
    durationInFrames: duration,
    component: "core/group",
    props: { ...props, tracks: clip.tracks } satisfies GroupProps,
    nodeId,
  };
  return {
    type: "clip",
    durationInFrames: duration,
    tracks: [{ id: `${nodeId}:t`, items: [group] }],
    audio: clip.audio,
    recording: clip.recording,
  };
}

export const shiftAudio = (audio: AudioItem[], offset: number, prefix: string): AudioItem[] =>
  audio.map((a) => ({ ...a, id: `${prefix}/${a.id}`, from: a.from + offset }));

export function sequenceClips(
  nodeId: string,
  clips: ClipValue[],
  transitions: Array<TransitionRef | undefined> = [],
): ClipValue {
  if (clips.length === 0) return emptyClip();
  const items: TimelineItem[] = [];
  const audio: AudioItem[] = [];
  let cursor = 0;
  clips.forEach((clip, i) => {
    let enter = i > 0 ? transitions[i] : undefined;
    if (enter) {
      const max = Math.min(clips[i - 1].durationInFrames, clip.durationInFrames) - 1;
      const d = Math.max(0, Math.min(enter.durationInFrames, max));
      enter = d > 0 ? { ...enter, durationInFrames: d } : undefined;
    }
    const from = Math.max(0, cursor - (enter?.durationInFrames ?? 0));
    if (enter && items[i - 1]) items[i - 1] = { ...items[i - 1], exit: enter };
    items.push({
      id: `${nodeId}:s${i}`,
      from,
      durationInFrames: clip.durationInFrames,
      component: "core/group",
      props: { tracks: clip.tracks } satisfies GroupProps,
      enter,
    });
    audio.push(...shiftAudio(clip.audio, from, `${nodeId}:s${i}`));
    cursor = from + clip.durationInFrames;
  });
  return {
    type: "clip",
    durationInFrames: Math.max(1, cursor),
    tracks: [{ id: `${nodeId}:seq`, items }],
    audio,
    recording: clips.find((c) => c.recording)?.recording,
  };
}

export function stackClips(nodeId: string, clips: ClipValue[], offsets: number[] = []): ClipValue {
  if (clips.length === 0) return emptyClip();
  const tracks: TimelineTrack[] = clips.map((clip, i) => ({
    id: `${nodeId}:l${i}`,
    items: [
      {
        id: `${nodeId}:l${i}:g`,
        from: offsets[i] ?? 0,
        durationInFrames: clip.durationInFrames,
        component: "core/group",
        props: { tracks: clip.tracks } satisfies GroupProps,
      },
    ],
  }));
  const duration = Math.max(...clips.map((c, i) => c.durationInFrames + (offsets[i] ?? 0)));
  return {
    type: "clip",
    durationInFrames: duration,
    tracks,
    audio: clips.flatMap((c, i) => shiftAudio(c.audio, offsets[i] ?? 0, `${nodeId}:l${i}`)),
    recording: clips.find((c) => c.recording)?.recording,
  };
}

export function mapItems(
  tracks: TimelineTrack[],
  fn: (item: TimelineItem) => TimelineItem,
): TimelineTrack[] {
  return tracks.map((t) => ({
    ...t,
    items: t.items.map((item) => {
      const mapped = fn(item);
      if (mapped.component === "core/group" && Array.isArray((mapped.props as GroupProps).tracks)) {
        const props = mapped.props as GroupProps;
        return { ...mapped, props: { ...props, tracks: mapItems(props.tracks, fn) } };
      }
      return mapped;
    }),
  }));
}

export function patchRecordings(
  clip: ClipValue,
  patch: (props: Record<string, unknown>) => Record<string, unknown>,
): ClipValue {
  return {
    ...clip,
    tracks: mapItems(clip.tracks, (item) =>
      item.component === "core/recording" ? { ...item, props: patch(item.props) } : item,
    ),
  };
}

export function hasRecording(clip: ClipValue): boolean {
  let found = false;
  mapItems(clip.tracks, (item) => {
    if (item.component === "core/recording") found = true;
    return item;
  });
  return found;
}

export function clipToSpec(clip: ClipValue, settings: WorkflowSettings): TimelineSpec {
  return {
    width: settings.width,
    height: settings.height,
    fps: settings.fps,
    durationInFrames: Math.max(1, clip.durationInFrames),
    background: settings.background ?? (settings.theme === "light" ? "#fafafa" : "#09090b"),
    tracks: clip.tracks,
    audio: clip.audio,
    fonts: settings.fonts,
  };
}

export type Span = { from: number; to: number };

export function nodeSpans(spec: TimelineSpec): Record<string, Span> {
  const spans: Record<string, Span> = {};
  const visit = (tracks: TimelineTrack[], offset: number, startFrom: number, limit: Span) => {
    for (const track of tracks) {
      for (const item of track.items) {
        const from = Math.max(limit.from, offset + item.from - startFrom);
        const to = Math.min(limit.to, offset + item.from - startFrom + item.durationInFrames);
        if (to <= from) continue;
        if (item.nodeId) {
          const s = spans[item.nodeId];
          spans[item.nodeId] = s
            ? { from: Math.min(s.from, from), to: Math.max(s.to, to) }
            : { from, to };
        }
        if (item.component === "core/group") {
          const p = item.props as GroupProps;
          visit(p.tracks ?? [], offset + item.from - startFrom, p.startFrom ?? 0, { from, to });
        }
      }
    }
  };
  visit(spec.tracks, 0, 0, { from: 0, to: spec.durationInFrames });
  return spans;
}

export function remapDuration(remap: Array<[number, number]>): number {
  return remap.length ? remap[remap.length - 1][0] : 0;
}

export function remapFrame(remap: Array<[number, number]>, frame: number): number {
  if (remap.length === 0) return frame;
  if (frame <= remap[0][0]) return remap[0][1];
  for (let i = 1; i < remap.length; i++) {
    const [o1, i1] = remap[i];
    const [o0, i0] = remap[i - 1];
    if (frame <= o1) return o1 === o0 ? i1 : i0 + ((frame - o0) / (o1 - o0)) * (i1 - i0);
  }
  const [oN, iN] = remap[remap.length - 1];
  return iN + (frame - oN);
}

export function remapFromSegments(
  innerDuration: number,
  segments: Array<[number, number, number]>,
): Array<[number, number]> {
  const out: Array<[number, number]> = [[0, 0]];
  let outPos = 0;
  let inPos = 0;
  const push = (inTo: number, rate: number) => {
    if (inTo <= inPos) return;
    outPos += (inTo - inPos) / rate;
    inPos = inTo;
    out.push([Math.round(outPos * 1000) / 1000, inPos]);
  };
  for (const [a, b, rate] of segments) {
    push(Math.min(a, innerDuration), 1);
    push(Math.min(b, innerDuration), Math.max(0.05, rate));
  }
  push(innerDuration, 1);
  return out;
}
