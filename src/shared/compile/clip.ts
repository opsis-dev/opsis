import type {
  AudioItem,
  EditHandles,
  TimelineItem,
  TimelineSpec,
  TimelineTrack,
  TransitionRef,
  VisualEnvelope,
} from "../schema/timeline";
import { hasKeys } from "../timeline/envelope";
import type { ClipValue, CompileValue, RecordingValue } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { clamp, lerp, round3, sampleKeyed } from "../math";
import { MIN_SPEED } from "../nodes/params";
import type { RecordingProps } from "./recording";

type Crop = { x: number; y: number; w: number; h: number };

export type Remap = Array<[number, number]>;

export type GroupProps = {
  tracks: TimelineTrack[];
  startFrom?: number;
  remap?: Remap;
  loopEvery?: number;
  crop?: Crop;
  wrapper?: { component: string; props: Record<string, unknown> };
  background?: string;
  design?: { width: number; height: number };
};

export const groupOf = (item: TimelineItem): GroupProps | null =>
  item.component === "core/group" ? (item.props as GroupProps) : null;

export const recordingOf = (item: TimelineItem): RecordingProps | null =>
  item.component === "core/recording" ? (item.props as RecordingProps) : null;

export const itemHasSound = (item: TimelineItem): boolean =>
  item.component === "core/video" && item.props.muted === false;

export const isClip = (v: CompileValue | undefined): v is ClipValue => v?.type === "clip";

export const EMPTY_CLIP_FRAMES = 30;

export function emptyClip(durationInFrames = 1): ClipValue {
  return { type: "clip", durationInFrames, tracks: [], audio: [] };
}

const wholeFrames = (frames: number): number => Math.max(1, Math.round(frames));

const firstRecording = (clips: ClipValue[]): RecordingValue | undefined =>
  clips.find((c) => c.recording)?.recording;

const makeClip = (
  durationInFrames: number,
  tracks: TimelineTrack[],
  audio: AudioItem[],
  recording: RecordingValue | undefined,
): ClipValue => ({ type: "clip", durationInFrames, tracks, audio, recording });

export type ItemExtra = {
  recording?: RecordingValue;
  audio?: AudioItem[];
  handles?: EditHandles;
  envelope?: VisualEnvelope;
};

export function itemClip(
  nodeId: string,
  component: string,
  props: Record<string, unknown>,
  durationInFrames: number,
  extra: ItemExtra = {},
): ClipValue {
  const duration = wholeFrames(durationInFrames);
  const item: TimelineItem = {
    id: `${nodeId}:i`,
    from: 0,
    durationInFrames: duration,
    component,
    props,
    nodeId,
    ...(extra.handles ? { handles: extra.handles } : {}),
    ...(hasKeys(extra.envelope) ? { envelope: extra.envelope } : {}),
  };
  return makeClip(
    duration,
    [{ id: `${nodeId}:t`, items: [item] }],
    extra.audio ?? [],
    extra.recording,
  );
}

export const placeholderClip = (
  nodeId: string,
  text: { title: string; subtitle: string },
  durationInFrames: number,
  extra: ItemExtra = {},
): ClipValue => itemClip(nodeId, "core/placeholder", text, durationInFrames, extra);

export function wrapClip(
  clip: ClipValue,
  nodeId: string,
  props: Omit<GroupProps, "tracks"> = {},
  durationInFrames = clip.durationInFrames,
): ClipValue {
  const duration = wholeFrames(durationInFrames);
  const group: TimelineItem = {
    id: `${nodeId}:g`,
    from: 0,
    durationInFrames: duration,
    component: "core/group",
    props: { ...props, tracks: clip.tracks } satisfies GroupProps,
    nodeId,
  };
  return makeClip(duration, [{ id: `${nodeId}:t`, items: [group] }], clip.audio, clip.recording);
}

export function withHandles(
  clip: ClipValue,
  handles: EditHandles,
  envelope?: VisualEnvelope,
): ClipValue {
  if (clip.tracks.length !== 1 || clip.tracks[0].items.length !== 1) return clip;
  const [track] = clip.tracks;
  const item = track.items[0];
  return {
    ...clip,
    tracks: [
      {
        ...track,
        items: [
          {
            ...item,
            handles: { ...item.handles, ...handles },
            ...(hasKeys(envelope) ? { envelope } : {}),
          },
        ],
      },
    ],
  };
}

const shiftAudio = (audio: AudioItem[], offset: number, prefix: string): AudioItem[] =>
  audio.map((a) => ({ ...a, id: `${prefix}/${a.id}`, from: a.from + offset }));

function resolveEnter(
  previous: ClipValue,
  clip: ClipValue,
  enter: TransitionRef | undefined,
): TransitionRef | undefined {
  if (!enter) return undefined;
  const longest = enter.hold
    ? enter.durationInFrames
    : Math.min(previous.durationInFrames, clip.durationInFrames) - 1;
  const frames = clamp(longest, 0, enter.durationInFrames);
  return frames > 0 ? { ...enter, durationInFrames: frames } : undefined;
}

const REMAP_START: Remap = [[0, 0]];

function holdLastFrame(item: TimelineItem, clipFrames: number, frames: number): TimelineItem {
  const last = clipFrames - 1;
  const end = item.durationInFrames - 1;
  const remap: Remap = [
    ...(groupOf(item)?.remap ?? REMAP_START),
    [end, last],
    [end + frames, last],
  ];
  return {
    ...item,
    durationInFrames: item.durationInFrames + frames,
    props: { ...item.props, remap },
  };
}

function placeClip(
  id: string,
  clip: ClipValue,
  at: { from: number; lead: number },
  enter: TransitionRef | undefined,
  slot: TimelineItem["slot"],
): TimelineItem {
  const { from, lead } = at;
  const remap: Remap = [
    [0, 0],
    [lead, 0],
  ];
  return {
    id,
    from,
    durationInFrames: clip.durationInFrames + lead,
    component: "core/group",
    props: (lead ? { tracks: clip.tracks, remap } : { tracks: clip.tracks }) satisfies GroupProps,
    enter,
    ...(slot ? { slot } : {}),
  };
}

export function sequenceClips(
  nodeId: string,
  clips: ClipValue[],
  transitions: Array<TransitionRef | undefined> = [],
  opts: { ports?: string[]; slotNode?: string } = {},
): ClipValue {
  if (clips.length === 0) return emptyClip();
  const items: TimelineItem[] = [];
  const audio: AudioItem[] = [];
  let cursor = 0;
  clips.forEach((clip, i) => {
    const previous = i > 0 ? clips[i - 1] : undefined;
    const enter = previous ? resolveEnter(previous, clip, transitions[i]) : undefined;
    const frames = enter?.durationInFrames ?? 0;
    const held = !!enter?.hold;
    if (previous && enter) {
      const before = held
        ? holdLastFrame(items[i - 1], previous.durationInFrames, frames)
        : items[i - 1];
      items[i - 1] = { ...before, exit: enter };
    }
    const from = held ? cursor : Math.max(0, cursor - frames);
    const lead = held ? frames : 0;
    const port = opts.ports?.[i];
    const slot = port ? { nodeId: opts.slotNode ?? nodeId, port } : undefined;
    items.push(placeClip(`${nodeId}:s${i}`, clip, { from, lead }, enter, slot));
    audio.push(...shiftAudio(clip.audio, from + lead, `${nodeId}:s${i}`));
    cursor = from + lead + clip.durationInFrames;
  });
  return makeClip(
    Math.max(1, cursor),
    [{ id: `${nodeId}:seq`, items }],
    audio,
    firstRecording(clips),
  );
}

export function stackClips(
  nodeId: string,
  clips: ClipValue[],
  offsets: number[] = [],
  opts: { ports?: string[]; move?: (i: number) => EditHandles["move"] } = {},
): ClipValue {
  if (clips.length === 0) return emptyClip();
  const offset = (i: number) => offsets[i] ?? 0;
  const tracks: TimelineTrack[] = clips.map((clip, i) => {
    const move = opts.move?.(i);
    const port = opts.ports?.[i];
    return {
      id: `${nodeId}:l${i}`,
      items: [
        {
          id: `${nodeId}:l${i}:g`,
          from: offset(i),
          durationInFrames: clip.durationInFrames,
          component: "core/group",
          props: { tracks: clip.tracks } satisfies GroupProps,
          ...(move ? { handles: { move } } : {}),
          ...(port ? { slot: { nodeId, port } } : {}),
        },
      ],
    };
  });
  return makeClip(
    Math.max(...clips.map((c, i) => c.durationInFrames + offset(i))),
    tracks,
    clips.flatMap((c, i) => shiftAudio(c.audio, offset(i), `${nodeId}:l${i}`)),
    firstRecording(clips),
  );
}

export function mapItems(
  tracks: TimelineTrack[],
  fn: (item: TimelineItem) => TimelineItem,
): TimelineTrack[] {
  return tracks.map((t) => ({
    ...t,
    items: t.items.map((item) => {
      const mapped = fn(item);
      const group = groupOf(mapped);
      return group
        ? { ...mapped, props: { ...group, tracks: mapItems(group.tracks, fn) } }
        : mapped;
    }),
  }));
}

export function forEachItem(tracks: TimelineTrack[], fn: (item: TimelineItem) => void): void {
  for (const track of tracks)
    for (const item of track.items) {
      fn(item);
      const group = groupOf(item);
      if (group) forEachItem(group.tracks, fn);
    }
}

export function patchRecordings(
  clip: ClipValue,
  patch: (props: RecordingProps) => Partial<RecordingProps>,
): ClipValue {
  return {
    ...clip,
    tracks: mapItems(clip.tracks, (item) => {
      const props = recordingOf(item);
      return props ? { ...item, props: patch(props) } : item;
    }),
  };
}

export function recordingProps(clip: ClipValue): RecordingProps[] {
  const out: RecordingProps[] = [];
  forEachItem(clip.tracks, (item) => {
    const props = recordingOf(item);
    if (props) out.push(props);
  });
  return out;
}

const THEME_BACKGROUNDS: Record<WorkflowSettings["theme"], string> = {
  light: "#fafafa",
  dark: "#09090b",
};

export function clipToSpec(clip: ClipValue, settings: WorkflowSettings): TimelineSpec {
  return {
    width: settings.width,
    height: settings.height,
    fps: settings.fps,
    durationInFrames: Math.max(1, clip.durationInFrames),
    background: settings.background ?? THEME_BACKGROUNDS[settings.theme],
    tracks: clip.tracks,
    audio: clip.audio,
    fonts: settings.fonts,
  };
}

export type FrameRange = { from: number; to: number };

export function nodeSpans(spec: TimelineSpec): Record<string, FrameRange> {
  const spans: Record<string, FrameRange> = {};
  const visit = (tracks: TimelineTrack[], offset: number, startFrom: number, limit: FrameRange) => {
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
        const group = groupOf(item);
        if (group)
          visit(group.tracks, offset + item.from - startFrom, group.startFrom ?? 0, { from, to });
      }
    }
  };
  visit(spec.tracks, 0, 0, { from: 0, to: spec.durationInFrames });
  return spans;
}

export const lerpSegment = (
  [x0, y0]: readonly [number, number],
  [x1, y1]: readonly [number, number],
  x: number,
): number => y0 + ((x - x0) / (x1 - x0)) * (y1 - y0);

export function remapDuration(remap: Remap): number {
  return remap.length ? remap[remap.length - 1][0] : 0;
}

export function remapFrame(remap: Remap, frame: number): number {
  const last = remap.at(-1);
  if (!last) return frame;
  if (frame > last[0]) return last[1] + (frame - last[0]);
  return (
    sampleKeyed(
      remap,
      frame,
      ([out]) => out,
      ([, inner]) => inner,
      (a, b, t) => lerp(a[1], b[1], t),
    ) ?? frame
  );
}

export function remapFromSegments(
  innerDuration: number,
  segments: Array<[number, number, number]>,
): Remap {
  const out: Remap = [[0, 0]];
  let outPos = 0;
  let inPos = 0;
  const push = (inTo: number, rate: number) => {
    if (inTo <= inPos) return;
    outPos += (inTo - inPos) / rate;
    inPos = inTo;
    out.push([round3(outPos), inPos]);
  };
  for (const [a, b, rate] of segments) {
    push(Math.min(a, innerDuration), 1);
    push(Math.min(b, innerDuration), Math.max(MIN_SPEED, rate));
  }
  push(innerDuration, 1);
  return out;
}
