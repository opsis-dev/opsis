import { z } from "zod";
import { findById } from "../lists";
import { clamp } from "../math";
import { parseOncePer } from "../memo";
import type { InputTake } from "../nodes/card";
import { rectSchema, type Viewport, viewportSchema } from "../schema/actions";
import {
  elementRefSchema,
  timelineTrackSchema,
  type AudioItem,
  type EditHandles,
  type Keyframe,
  type TimelineItem,
  type TimelineSpec,
  type TimelineTrack,
  type TransitionRef,
  type VisualEnvelope,
} from "../schema/timeline";
import type { ClipValue, CompileValue, RecordingValue } from "../schema/values";
import { THEME_BACKGROUNDS, type WorkflowSettings } from "../schema/workflow";
import { hasKeys } from "../timeline/envelope";
import { atLeastOneFrame } from "../timeline/time";
import { recordingPropsSchema, type RecordingProps } from "./recording";
import { inverseRemap, remapSchema, type Remap } from "./remap";

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

const videoPropsSchema = z.object({
  src: z.string(),
  loop: z.boolean().default(false),
  muted: z.boolean().default(true),
  volume: z.number().default(1),
  trimBeforeMs: z.number().default(0),
});
export type VideoProps = z.infer<typeof videoPropsSchema>;

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

const readVideo = parseOncePer(videoPropsSchema);

export const videoOf = (item: Pick<TimelineItem, "component" | "props">): VideoProps | null =>
  item.component === "core/video" ? readVideo(item.props) : null;

type ItemPlacement = Omit<TimelineItem, "component" | "props">;

export const groupItem = (placement: ItemPlacement, props: GroupProps): TimelineItem => ({
  ...placement,
  component: "core/group",
  props,
});

export const itemHasSound = (item: TimelineItem): boolean => videoOf(item)?.muted === false;

export const soundNodeId = (item: TimelineItem): string | undefined =>
  itemHasSound(item) ? item.nodeId : undefined;

export const isClip = (v: CompileValue | undefined): v is ClipValue => v?.type === "clip";

export const EMPTY_CLIP_FRAMES = 30;

export function emptyClip(durationInFrames = 1): ClipValue {
  return makeClip({ durationInFrames, tracks: [], audio: [] });
}

const firstRecording = (clips: ClipValue[]): RecordingValue | undefined =>
  clips.find((c) => c.recording)?.recording;

const makeClip = (parts: Omit<ClipValue, "type">): ClipValue => ({ type: "clip", ...parts });

const oneTrack = (id: string, item: TimelineItem): TimelineTrack[] => [
  { id: `${id}:t`, items: [item] },
];

export type ItemExtra = {
  recording?: RecordingValue;
  audio?: AudioItem[];
  handles?: EditHandles;
  envelope?: VisualEnvelope;
  design?: Viewport;
};

export function itemClip<C extends string>(
  nodeId: string,
  component: C,
  props: PropsFor<C>,
  durationInFrames: number,
  extra: ItemExtra = {},
): ClipValue {
  const duration = atLeastOneFrame(durationInFrames);
  const item: TimelineItem = {
    id: `${nodeId}:i`,
    from: 0,
    durationInFrames: duration,
    component,
    props,
    nodeId,
    ...(extra.handles ? { handles: extra.handles } : {}),
    ...(hasKeys(extra.envelope) ? { envelope: extra.envelope } : {}),
    ...(extra.design ? { design: extra.design } : {}),
  };
  return makeClip({
    durationInFrames: duration,
    tracks: oneTrack(nodeId, item),
    audio: extra.audio ?? [],
    recording: extra.recording,
  });
}

function cutSoundStart(a: AudioItem, by: number): AudioItem | null {
  if (by <= 0) return a;
  if (a.durationInFrames !== undefined && a.durationInFrames <= by) return null;
  const sourceCut = Math.round(by * (a.rate ?? 1));
  if (a.lengthFrames !== undefined && a.lengthFrames <= sourceCut) return null;
  return {
    ...a,
    from: a.from + by,
    trimBefore: (a.trimBefore ?? 0) + sourceCut,
    ...(a.durationInFrames === undefined ? {} : { durationInFrames: a.durationInFrames - by }),
    ...(a.lengthFrames === undefined ? {} : { lengthFrames: a.lengthFrames - sourceCut }),
  };
}

function fitWithin(a: AudioItem, end: number): AudioItem | null {
  const placed = cutSoundStart(a, -a.from);
  if (!placed || placed.from >= end) return null;
  const room = end - placed.from;
  return {
    ...placed,
    durationInFrames: Math.min(placed.durationInFrames ?? room, room),
  };
}

const scaleKeys = (keys: Keyframe[] | undefined, by: number) =>
  keys?.map((k) => ({ ...k, frame: Math.round(k.frame * by) }));

function speedSound(a: AudioItem, innerPerOuter: number): AudioItem {
  const outer = (frames: number | undefined) =>
    frames === undefined ? undefined : Math.max(1, Math.round(frames / innerPerOuter));
  return {
    ...a,
    from: Math.round(a.from / innerPerOuter),
    rate: (a.rate ?? 1) * innerPerOuter,
    ...(a.durationInFrames === undefined ? {} : { durationInFrames: outer(a.durationInFrames) }),
    ...(a.fadeInFrames ? { fadeInFrames: outer(a.fadeInFrames) } : {}),
    ...(a.fadeOutFrames ? { fadeOutFrames: outer(a.fadeOutFrames) } : {}),
    ...(a.envelope
      ? {
          envelope: {
            volume: scaleKeys(a.envelope.volume, 1 / innerPerOuter),
            pan: scaleKeys(a.envelope.pan, 1 / innerPerOuter),
          },
        }
      : {}),
  };
}

const uniformRate = (remap: Remap): number | null => {
  const [[x0, y0], ...rest] = remap;
  const last = rest.at(-1);
  if (rest.length !== 1 || x0 !== 0 || y0 !== 0 || !last || last[0] <= 0) return null;
  return last[1] / last[0];
};

export function wrapSounds(
  audio: AudioItem[],
  props: Pick<GroupProps, "startFrom" | "loopEvery" | "remap">,
  duration: number,
): AudioItem[] {
  let items = audio.flatMap(
    (a) => cutSoundStart({ ...a, from: a.from - (props.startFrom ?? 0) }, 0) ?? [],
  );
  const loop = props.loopEvery ?? 0;
  if (loop > 0) {
    const count = Math.ceil(duration / loop);
    items = Array.from({ length: count }, (_, k) =>
      items.flatMap((a) => {
        const inLoop = fitWithin(a, loop);
        if (!inLoop) return [];
        const copy = { ...inLoop, from: inLoop.from + k * loop };
        return k === 0 ? [copy] : [{ ...copy, id: `${a.id}:r${k}`, handles: undefined }];
      }),
    ).flat();
  }
  const { remap } = props;
  if (remap?.length) {
    const rate = uniformRate(remap);
    items =
      rate === null
        ? items.map((a) => ({ ...a, from: Math.round(inverseRemap(remap, a.from, "start")) }))
        : items.map((a) => speedSound(a, rate));
  }
  return items.flatMap((a) => fitWithin(a, duration) ?? []);
}

export function wrapClip(
  clip: ClipValue,
  nodeId: string,
  props: Omit<GroupProps, "tracks"> = {},
  durationInFrames = clip.durationInFrames,
): ClipValue {
  const duration = atLeastOneFrame(durationInFrames);
  const group = groupItem(
    { id: `${nodeId}:g`, from: 0, durationInFrames: duration, nodeId },
    { ...props, tracks: clip.tracks },
  );
  return makeClip({
    durationInFrames: duration,
    tracks: oneTrack(nodeId, group),
    audio: wrapSounds(clip.audio, props, duration),
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

const overlaps = (t: TransitionRef | undefined): boolean => !!t && !t.hold;

function shareOf(clip: ClipValue, otherSide: TransitionRef | undefined): number {
  return overlaps(otherSide) ? Math.floor(clip.durationInFrames / 2) : clip.durationInFrames - 1;
}

function resolveEnters(
  clips: ClipValue[],
  transitions: Array<TransitionRef | undefined>,
): Array<TransitionRef | undefined> {
  return clips.map((clip, i) => {
    const enter = i > 0 ? transitions[i] : undefined;
    if (!enter) return undefined;
    const longest = enter.hold
      ? enter.durationInFrames
      : Math.min(
          shareOf(clips[i - 1], i > 1 ? transitions[i - 1] : undefined),
          shareOf(clip, transitions[i + 1]),
        );
    const frames = clamp(longest, 0, enter.durationInFrames);
    return frames > 0 ? { ...enter, durationInFrames: frames } : undefined;
  });
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
  const enters = resolveEnters(clips, transitions);
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
  const { move: _move, ...kept } = handles ?? {};
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
    tracks: oneTrack(id, whole),
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
    tracks: oneTrack(nodeId, group),
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
  patch: (props: RecordingProps) => RecordingProps,
): ClipValue {
  return {
    ...clip,
    tracks: mapItems(clip.tracks, (item) => {
      const props = recordingOf(item);
      return props ? { ...item, props: patch(props) } : item;
    }),
  };
}

export function clipFrameOfRecording(
  tracks: TimelineTrack[],
  content: (recording: RecordingProps) => number,
): number | null {
  for (const track of tracks)
    for (const item of track.items) {
      const recording = recordingOf(item);
      if (recording) return item.from + content(recording);
      const group = groupOf(item);
      const inner = group ? clipFrameOfRecording(group.tracks, content) : null;
      if (group && inner !== null) {
        const local = inner - (group.startFrom ?? 0);
        const outer = group.remap?.length ? inverseRemap(group.remap, local, "start") : local;
        return item.from + outer;
      }
    }
  return null;
}

export function inputTake(clip: ClipValue): InputTake | undefined {
  const log = clip.recording?.log;
  if (log) return { durationMs: log.durationMs, cursorTrack: !!log.cursorTrack?.length };
  let placeholder = false;
  walkItems(clip.tracks, (item) => {
    if (item.component === "core/placeholder") placeholder = true;
  });
  return placeholder ? undefined : null;
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
