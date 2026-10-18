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
import { wholeFrames } from "../timeline/time";
import type { ClipValue, CompileValue, RecordingValue } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { clamp } from "../math";
import type { RecordingProps } from "./recording";
import type { Remap } from "./remap";

export type Crop = { x: number; y: number; w: number; h: number };

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

export const groupItem = (
  { id, from, durationInFrames, ...rest }: ItemPlacement,
  props: GroupProps,
): TimelineItem => ({ id, from, durationInFrames, component: "core/group", props, ...rest });

export const itemHasSound = (item: TimelineItem): boolean =>
  item.component === "core/video" && item.props.muted === false;

export const soundNodeId = (item: TimelineItem): string | undefined =>
  itemHasSound(item) ? item.nodeId : undefined;

export const isClip = (v: CompileValue | undefined): v is ClipValue => v?.type === "clip";

export const EMPTY_CLIP_FRAMES = 30;

export function emptyClip(durationInFrames = 1): ClipValue {
  return { type: "clip", durationInFrames, tracks: [], audio: [] };
}

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
  const duration = wholeFrames(durationInFrames, 1);
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
  const duration = wholeFrames(durationInFrames, 1);
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

function withExit(item: TimelineItem, clip: ClipValue, exit: TransitionRef | undefined) {
  if (!exit) return item;
  const before = exit.hold
    ? holdLastFrame(item, clip.durationInFrames, exit.durationInFrames)
    : item;
  return { ...before, exit };
}

type Placement = { from: number; lead: number };

function sequencePlacements(
  clips: ClipValue[],
  enters: Array<TransitionRef | undefined>,
): { at: Placement[]; end: number } {
  const at: Placement[] = [];
  let end = 0;
  clips.forEach((clip, i) => {
    const enter = enters[i];
    const frames = enter?.durationInFrames ?? 0;
    const held = !!enter?.hold;
    const from = held ? end : Math.max(0, end - frames);
    const lead = held ? frames : 0;
    at.push({ from, lead });
    end = from + lead + clip.durationInFrames;
  });
  return { at, end };
}

export function sequenceClips(
  nodeId: string,
  clips: ClipValue[],
  transitions: Array<TransitionRef | undefined> = [],
  opts: { ports?: string[]; slotNode?: string } = {},
): ClipValue {
  if (clips.length === 0) return emptyClip();
  const enters = clips.map((clip, i) =>
    i > 0 ? resolveEnter(clips[i - 1], clip, transitions[i]) : undefined,
  );
  const { at, end } = sequencePlacements(clips, enters);
  const items = clips.map((clip, i) => {
    const port = opts.ports?.[i];
    const slot = port ? { nodeId: opts.slotNode ?? nodeId, port } : undefined;
    const item = placeClip(`${nodeId}:s${i}`, clip, at[i], enters[i], slot);
    return withExit(item, clip, enters[i + 1]);
  });
  const audio = clips.flatMap((clip, i) =>
    shiftAudio(clip.audio, at[i].from + at[i].lead, `${nodeId}:s${i}`),
  );
  return makeClip(Math.max(1, end), [{ id: `${nodeId}:seq`, items }], audio, firstRecording(clips));
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

export function walkItems(tracks: TimelineTrack[], visit: (item: TimelineItem) => void): void {
  for (const track of tracks)
    for (const item of track.items) {
      visit(item);
      const group = groupOf(item);
      if (group) walkItems(group.tracks, visit);
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

export const THEME_BACKGROUNDS: Record<WorkflowSettings["theme"], string> = {
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
