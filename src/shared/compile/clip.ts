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
import { clamp } from "../math";
import type { RecordingProps } from "./recording";
import type { Remap } from "./remap";

type Crop = { x: number; y: number; w: number; h: number };

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

type BuiltInProps = { "core/group": GroupProps; "core/recording": RecordingProps };

const propsOf = <C extends keyof BuiltInProps>(
  item: TimelineItem,
  component: C,
): BuiltInProps[C] | null =>
  item.component === component ? (item.props as BuiltInProps[C]) : null;

export const groupOf = (item: TimelineItem): GroupProps | null => propsOf(item, "core/group");

export const recordingOf = (item: TimelineItem): RecordingProps | null =>
  propsOf(item, "core/recording");

type ItemPlacement = Omit<TimelineItem, "component" | "props">;

const builtInItem = <C extends keyof BuiltInProps>(
  { id, from, durationInFrames, ...rest }: ItemPlacement,
  component: C,
  props: BuiltInProps[C],
): TimelineItem => ({ id, from, durationInFrames, component, props, ...rest });

export const groupItem = (placement: ItemPlacement, props: GroupProps): TimelineItem =>
  builtInItem(placement, "core/group", props);

export const itemHasSound = (item: TimelineItem): boolean =>
  item.component === "core/video" && item.props.muted === false;

export const soundNodeId = (item: TimelineItem): string | undefined =>
  itemHasSound(item) ? item.nodeId : undefined;

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

export const recordingClip = (
  nodeId: string,
  props: RecordingProps,
  durationInFrames: number,
  extra: ItemExtra = {},
): ClipValue => itemClip(nodeId, "core/recording", props, durationInFrames, extra);

export function wrapClip(
  clip: ClipValue,
  nodeId: string,
  props: Omit<GroupProps, "tracks"> = {},
  durationInFrames = clip.durationInFrames,
): ClipValue {
  const duration = wholeFrames(durationInFrames);
  const group = groupItem(
    { id: `${nodeId}:g`, from: 0, durationInFrames: duration, nodeId },
    { ...props, tracks: clip.tracks },
  );
  return makeClip(duration, [{ id: `${nodeId}:t`, items: [group] }], clip.audio, clip.recording);
}

export function withHandles(
  clip: ClipValue,
  handles: EditHandles,
  envelope?: VisualEnvelope,
): ClipValue {
  if (clip.tracks.length !== 1 || clip.tracks[0].items.length !== 1) return clip;
  const [track] = clip.tracks;
  const [item] = track.items;
  const patched: TimelineItem = {
    ...item,
    handles: { ...item.handles, ...handles },
    ...(hasKeys(envelope) ? { envelope } : {}),
  };
  return { ...clip, tracks: [{ ...track, items: [patched] }] };
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
  return groupItem(
    { id, from, durationInFrames: clip.durationInFrames + lead, enter, ...(slot ? { slot } : {}) },
    lead ? { tracks: clip.tracks, remap } : { tracks: clip.tracks },
  );
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
        groupItem(
          {
            id: `${nodeId}:l${i}:g`,
            from: offset(i),
            durationInFrames: clip.durationInFrames,
            ...(move ? { handles: { move } } : {}),
            ...(port ? { slot: { nodeId, port } } : {}),
          },
          { tracks: clip.tracks },
        ),
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

export type FrameRange = { from: number; to: number };

const EVERYWHERE: FrameRange = { from: -Infinity, to: Infinity };

export function walkItems(
  tracks: TimelineTrack[],
  visit: (item: TimelineItem, span: FrameRange) => void,
  within: FrameRange = EVERYWHERE,
  offset = 0,
): void {
  for (const track of tracks)
    for (const item of track.items) {
      const start = offset + item.from;
      const span = {
        from: Math.max(within.from, start),
        to: Math.min(within.to, start + item.durationInFrames),
      };
      visit(item, span);
      const group = groupOf(item);
      if (group) walkItems(group.tracks, visit, span, start - (group.startFrom ?? 0));
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
  walkItems(clip.tracks, (item) => {
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

export function nodeSpans(spec: TimelineSpec): Record<string, FrameRange> {
  const spans: Record<string, FrameRange> = {};
  const film = { from: 0, to: spec.durationInFrames };
  walkItems(
    spec.tracks,
    ({ nodeId }, { from, to }) => {
      if (to <= from || !nodeId) return;
      const s = spans[nodeId];
      spans[nodeId] = s ? { from: Math.min(s.from, from), to: Math.max(s.to, to) } : { from, to };
    },
    film,
  );
  return spans;
}
