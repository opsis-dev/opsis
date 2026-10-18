import { z } from "zod";
import { findById } from "../lists";
import { clamp } from "../math";
import { parseOncePer } from "../memo";
import { rectSchema, viewportSchema } from "../schema/actions";
import {
  elementRefSchema,
  timelineTrackSchema,
  type AudioItem,
  type EditHandles,
  type TimelineItem,
  type TimelineSpec,
  type TimelineTrack,
  type TransitionRef,
  type VisualEnvelope,
} from "../schema/timeline";
import type { ClipValue, CompileValue, RecordingValue } from "../schema/values";
import { THEME_BACKGROUNDS, type WorkflowSettings } from "../schema/workflow";
import { hasKeys } from "../timeline/envelope";
import { wholeFrames } from "../timeline/time";
import { recordingPropsSchema, type RecordingProps } from "./recording";
import { remapSchema, type Remap } from "./remap";

const sequencerPropsSchema = elementRefSchema.extend({
  backgrounds: z.array(z.string().nullable()),
});
export type SequencerProps = z.infer<typeof sequencerPropsSchema>;

const groupPropsSchema = z.object({
  tracks: z.array(timelineTrackSchema),
  startFrom: z.number().optional(),
  remap: remapSchema.optional(),
  loopEvery: z.number().optional(),
  crop: rectSchema.optional(),
  wrapper: elementRefSchema.optional(),
  background: z.string().optional(),
  design: viewportSchema.optional(),
  sequencer: sequencerPropsSchema.optional(),
});
export type GroupProps = z.infer<typeof groupPropsSchema>;

type PlaceholderProps = { title: string; subtitle: string };

type BuiltInProps = {
  "core/group": GroupProps;
  "core/recording": RecordingProps;
  "core/placeholder": PlaceholderProps;
};

type PropsFor<C extends string> = C extends keyof BuiltInProps
  ? BuiltInProps[C]
  : Record<string, unknown>;

const readGroup = parseOncePer(groupPropsSchema);

const readRecording = parseOncePer(recordingPropsSchema);

export const groupOf = (item: TimelineItem): GroupProps | null =>
  item.component === "core/group" ? readGroup(item.props) : null;

export const recordingOf = (item: TimelineItem): RecordingProps | null =>
  item.component === "core/recording" ? readRecording(item.props) : null;

type ItemPlacement = Omit<TimelineItem, "component" | "props">;

export const groupItem = (placement: ItemPlacement, props: GroupProps): TimelineItem => ({
  ...placement,
  component: "core/group",
  props,
});

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

const makeClip = (parts: Omit<ClipValue, "type">): ClipValue => ({ type: "clip", ...parts });

export type ItemExtra = {
  recording?: RecordingValue;
  audio?: AudioItem[];
  handles?: EditHandles;
  envelope?: VisualEnvelope;
};

export function itemClip<C extends string>(
  nodeId: string,
  component: C,
  props: PropsFor<C>,
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
  return makeClip({
    durationInFrames: duration,
    tracks: [{ id: `${nodeId}:t`, items: [item] }],
    audio: extra.audio ?? [],
    recording: extra.recording,
  });
}

export const placeholderClip = (
  nodeId: string,
  text: PlaceholderProps,
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
  return makeClip({
    durationInFrames: duration,
    tracks: [{ id: `${nodeId}:t`, items: [group] }],
    audio: clip.audio,
    recording: clip.recording,
  });
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
  return makeClip({
    durationInFrames: Math.max(1, end),
    tracks: [{ id: `${nodeId}:seq`, items }],
    audio,
    recording: firstRecording(clips),
  });
}

const layerId = (nodeId: string, i: number) => `${nodeId}:l${i}`;

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
      id: layerId(nodeId, i),
      items: [
        groupItem(
          {
            id: `${layerId(nodeId, i)}:g`,
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
  return makeClip({
    durationInFrames: Math.max(...clips.map((c, i) => c.durationInFrames + offset(i))),
    tracks,
    audio: clips.flatMap((c, i) =>
      shiftAudio(c.audio, offset(i), layerId(nodeId, i)).map((a) => ({
        ...a,
        layerTrack: layerId(nodeId, i),
      })),
    ),
    recording: firstRecording(clips),
  });
}

export const markLayers = (clip: ClipValue): ClipValue => ({
  ...clip,
  tracks: clip.tracks.map((track) => ({ ...track, layer: true })),
});

const isLayered = (clip: ClipValue): boolean =>
  clip.tracks.length > 0 && clip.tracks.every((track) => track.layer);

export const sceneItemCount = (clip: ClipValue): number =>
  isLayered(clip) ? clip.tracks.length : 1;

type Scene = { tracks: TimelineTrack[]; audio: AudioItem[]; durationInFrames: number };

const layerStart = (track: TimelineTrack): number => track.items[0]?.from ?? 0;

function startWithScene(item: TimelineItem, by: number): TimelineItem {
  const { handles, ...rest } = item;
  const kept = { ...handles };
  delete kept.move;
  return { ...rest, from: item.from - by, ...(Object.keys(kept).length ? { handles: kept } : {}) };
}

function layersStartTogether(clip: ClipValue): Scene {
  const tracks = clip.tracks.map((track) => ({
    ...track,
    items: track.items.map((item) => startWithScene(item, layerStart(track))),
  }));
  const audio = clip.audio.map((a) => {
    const owner = findById(clip.tracks, a.layerTrack);
    return owner ? { ...a, from: a.from - layerStart(owner) } : a;
  });
  const longest = Math.max(
    ...tracks.flatMap((t) => t.items.map((i) => i.from + i.durationInFrames)),
  );
  return { tracks, audio, durationInFrames: Math.min(clip.durationInFrames, longest) };
}

function sceneOf(id: string, clip: ClipValue): Scene {
  if (isLayered(clip)) return layersStartTogether(clip);
  const whole = groupItem(
    { id: `${id}:i`, from: 0, durationInFrames: clip.durationInFrames },
    { tracks: clip.tracks },
  );
  return {
    tracks: [{ id: `${id}:t`, items: [whole] }],
    audio: clip.audio,
    durationInFrames: clip.durationInFrames,
  };
}

export function sceneSequence(
  nodeId: string,
  scenes: Array<{ port: string; clip: ClipValue }>,
  { gap, sequencer }: { gap: number; sequencer: SequencerProps },
): ClipValue {
  const built = scenes.map(({ port, clip }, i) => ({
    id: `${nodeId}:s${i}`,
    port,
    scene: sceneOf(`${nodeId}:s${i}`, clip),
  }));
  const starts: number[] = [];
  let end = 0;
  for (const { scene } of built) {
    starts.push(end);
    end += scene.durationInFrames + gap;
  }
  const duration = Math.max(1, end - gap);
  const placed = built.map(({ id, port, scene }, i) => ({
    item: groupItem(
      {
        id,
        from: starts[i],
        durationInFrames: scene.durationInFrames,
        slot: { nodeId, port },
      },
      { tracks: scene.tracks },
    ),
    audio: shiftAudio(scene.audio, starts[i], id),
  }));
  const group = groupItem(
    { id: `${nodeId}:g`, from: 0, durationInFrames: duration, nodeId },
    { tracks: [{ id: `${nodeId}:scenes`, items: placed.map((p) => p.item) }], sequencer },
  );
  return makeClip({
    durationInFrames: duration,
    tracks: [{ id: `${nodeId}:t`, items: [group] }],
    audio: placed.flatMap((p) => p.audio),
    recording: firstRecording(scenes.map((s) => s.clip)),
  });
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
