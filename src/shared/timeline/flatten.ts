import { groupOf, itemHasSound, recordingOf, type GroupProps } from "../compile/clip";
import { recordingSpeed, type RecordingProps } from "../compile/recording";
import { inverseRemap, remapFrame, type Remap, type RemapSide } from "../compile/remap";
import { MAX_REPEATS } from "../limits";
import { clamp, EPSILON } from "../math";
import type { ChannelKind } from "../mix/types";
import type { Action } from "../schema/actions";
import type {
  AudioItem,
  EditHandles,
  Slot,
  TimelineItem,
  TimelineSpec,
  TimelineTrack,
} from "../schema/timeline";
import { actionLabel } from "./action-labels";
import { audioBars } from "./audio-bars";
import { sortedCameraKeys } from "./camera-path";
import { overlaps, packAudio, packLanes } from "./lanes";
import { thumbnailOf } from "./thumbnails";
import { recordingMsToFrames } from "./time";

type Instance = {
  toAbs: (content: number, side: RemapSide) => number;
  lo: number;
  hi: number;
};

export type ChainLink = {
  itemId: string;
  nodeId?: string;
  handles?: EditHandles;
  slot?: Slot;
  rate: number;
  rateEnd: number;
  end: number;
  from: number;
  to: number;
};

export type BarThumbnail =
  | { kind: "filmstrip"; src: string; speed: number; trimMs: number }
  | { kind: "still"; src: string };

export type VideoBar = {
  type: "video";
  key: string;
  itemId: string;
  nodeId: string;
  component: string;
  props: Record<string, unknown>;
  thumbnail: BarThumbnail | null;
  from: number;
  to: number;
  lane: number;
  order: number;
  chain: ChainLink[];
  rate: number;
  contentStart: number;
  contentEnd: number;
  loop: number;
  localAt: (abs: number) => number;
  hasSound: boolean;
  sound: SoundWindow | null;
};

type SoundWindow = { from: number; to: number; contentStart: number };

export type AudioBar = {
  type: "audio";
  key: string;
  id: string;
  nodeId?: string;
  channel: string;
  src: string;
  role: ChannelKind;
  from: number;
  to: number;
  lane: number;
  trimBefore: number;
  loop: boolean;
  volume: number;
  handles?: EditHandles;
  item?: AudioItem;
  bar?: VideoBar;
};

export type TransitionMark = {
  key: string;
  itemKey: string;
  from: number;
  to: number;
  kind: string;
  lanes: number[];
  covers: number;
  slot?: Slot;
};

type ActionMark = {
  key: string;
  frame: number;
  kind: Action["kind"];
  label: string;
  nodeId?: string;
  barKey: string;
};

export type CameraKeyMark = {
  key: string;
  itemId: string;
  nodeId: string;
  index: number;
  frame: number;
};

export type FlatTimeline = {
  fps: number;
  durationInFrames: number;
  bars: VideoBar[];
  lanes: number;
  audio: AudioBar[];
  audioLanes: number;
  transitions: TransitionMark[];
  marks: ActionMark[];
  cameraKeys: CameraKeyMark[];
};

const MAX_LOOPS = MAX_REPEATS;

export type FrameRange = { from: number; to: number };

const nonEmptyRemap = (group: GroupProps): Remap | null =>
  group.remap?.length ? group.remap : null;

function groupInstances(
  item: TimelineItem,
  group: GroupProps,
  parent: Instance,
  absFrom: number,
  absTo: number,
): Instance[] {
  const startFrom = group.startFrom ?? 0;
  const remap = nonEmptyRemap(group);
  const loop = group.loopEvery && group.loopEvery > 0 ? group.loopEvery : 0;
  const dur = item.durationInFrames;
  const toOuter = (o: number, side: RemapSide) => parent.toAbs(item.from + o, side);
  const inv = (r: number, side: RemapSide) => (remap ? inverseRemap(remap, r, side) : r);
  const lo = Math.max(parent.lo, absFrom);
  const hi = Math.min(parent.hi, absTo);
  if (!loop) {
    return [
      {
        toAbs: (c: number, side: RemapSide) => toOuter(inv(c - startFrom, side), side),
        lo,
        hi,
      },
    ];
  }
  const maxR = remap ? remapFrame(remap, dur) : dur;
  const count = clamp(Math.ceil(maxR / loop), 1, MAX_LOOPS);
  return Array.from({ length: count }, (_, k) => {
    const start = toOuter(inv(k * loop, "start"), "start");
    const end = toOuter(inv((k + 1) * loop, "end"), "end");
    return {
      toAbs: (c: number, side: RemapSide) => toOuter(inv(c - startFrom + k * loop, side), side),
      lo: Math.max(lo, start),
      hi: Math.min(hi, end),
    };
  });
}

const BISECT_STEPS = 24;

function bisect(inst: Instance, a: number, b: number, abs: number): number {
  let lo = a;
  let hi = b;
  for (let i = 0; i < BISECT_STEPS; i++) {
    const mid = (lo + hi) / 2;
    if (inst.toAbs(mid, "start") < abs) lo = mid;
    else hi = mid;
  }
  return (lo + hi) / 2;
}

function rateAt(inst: Instance, content: number): number {
  const d = inst.toAbs(content + 1, "start") - inst.toAbs(content, "start");
  return d > EPSILON ? 1 / d : 1;
}

type Walk = {
  inst: Instance;
  path: string;
  chain: ChainLink[];
  loop: number;
  timing: Timing;
};

type Timing = "plain" | "held" | "changed";

const isFreezeOrPlay = (remap: Remap): boolean =>
  remap.every((point, k) => {
    if (k === 0) return true;
    const [o0, i0] = remap[k - 1];
    const slope = (point[1] - i0) / Math.max(EPSILON, point[0] - o0);
    return Math.abs(slope) < EPSILON || Math.abs(slope - 1) < EPSILON;
  });

function timingWith(timing: Timing, remap: Remap | null): Timing {
  if (!remap || timing === "changed") return timing;
  return isFreezeOrPlay(remap) ? "held" : "changed";
}

function soundWindow(
  p: Placed,
  from: number,
  to: number,
  startContent: number,
): SoundWindow | null {
  const { timing, inst } = p.walk;
  if (timing === "plain") return { from, to, contentStart: startContent - p.item.from };
  if (timing === "changed") return null;
  const playFrom = Math.max(from, inst.toAbs(startContent, "end"));
  const playTo = Math.min(to, inst.toAbs(p.contentEnd - 1, "start") + 1);
  if (playTo - playFrom < 1) return null;
  return { from: playFrom, to: playTo, contentStart: contentAt(p, playFrom) - p.item.from };
}

type Placed = {
  item: TimelineItem;
  walk: Walk;
  key: string;
  absStart: number;
  absEnd: number;
  from: number;
  to: number;
  contentEnd: number;
};

function place(item: TimelineItem, walk: Walk, track: number): Placed {
  const contentEnd = item.from + item.durationInFrames;
  const absStart = walk.inst.toAbs(item.from, "start");
  const absEnd = walk.inst.toAbs(contentEnd, "end");
  return {
    item,
    walk,
    key: `${walk.path}/${track}:${item.id}`,
    absStart,
    absEnd,
    from: Math.max(absStart, walk.inst.lo),
    to: Math.min(absEnd, walk.inst.hi),
    contentEnd,
  };
}

const contentAt = (p: Placed, abs: number) => bisect(p.walk.inst, p.item.from, p.contentEnd, abs);

function makeLink(p: Placed): ChainLink {
  const { item, walk, from, to } = p;
  return {
    itemId: item.id,
    ...(item.nodeId ? { nodeId: item.nodeId } : {}),
    ...(item.handles ? { handles: item.handles } : {}),
    ...(item.slot ? { slot: item.slot } : {}),
    rate: rateAt(walk.inst, item.from),
    rateEnd: rateAt(walk.inst, Math.max(item.from, p.contentEnd - 1)),
    end: to < p.absEnd - EPSILON ? contentAt(p, to) - item.from : item.durationInFrames,
    from,
    to,
  };
}

function transitionMark(p: Placed, previousLength: number): TransitionMark | null {
  const { item, from, key } = p;
  const enter = item.enter;
  if (!enter || enter.durationInFrames <= 0) return null;
  const to = Math.min(p.to, p.walk.inst.toAbs(item.from + enter.durationInFrames, "end"));
  const shorter = Math.min(p.absEnd - p.absStart, previousLength);
  return {
    key: `${key}:enter`,
    itemKey: key,
    lanes: [],
    from,
    to,
    kind: enter.kind,
    covers: enter.hold ? 0 : (to - from) / Math.max(EPSILON, shorter),
    ...(item.slot ? { slot: item.slot } : {}),
  };
}

function leafBar(p: Placed, link: ChainLink, order: number): VideoBar {
  const { item, walk, from, to } = p;
  const startContent = from > p.absStart + EPSILON ? contentAt(p, from) : item.from;
  const endContent = to < p.absEnd - EPSILON ? contentAt(p, to) : p.contentEnd;
  const hasSound = itemHasSound(item);
  return {
    type: "video",
    key: p.key,
    itemId: item.id,
    nodeId: item.nodeId ?? walk.chain.find((l) => l.nodeId)?.nodeId ?? item.id,
    component: item.component,
    props: item.props,
    thumbnail: thumbnailOf(item),
    from,
    to,
    lane: 0,
    order,
    chain: [link, ...walk.chain],
    rate: rateAt(walk.inst, startContent),
    contentStart: startContent - item.from,
    contentEnd: endContent - item.from,
    loop: walk.loop,
    localAt: (abs: number) => contentAt(p, abs) - item.from,
    hasSound,
    sound: hasSound ? soundWindow(p, from, to, startContent) : null,
  };
}

function recordingMarks(p: Placed, props: RecordingProps, fps: number): ActionMark[] {
  const { item, from, to, key } = p;
  const speed = recordingSpeed(props);
  return props.log.actions.flatMap((action) => {
    const label = actionLabel(action);
    if (label === null) return [];
    const local = recordingMsToFrames(action.t, speed, fps);
    const frame = p.walk.inst.toAbs(item.from + local, "start");
    if (frame < from || frame >= to) return [];
    return [
      {
        key: `${key}:${action.id}`,
        frame,
        kind: action.kind,
        label,
        ...(action.nodeId ? { nodeId: action.nodeId } : {}),
        barKey: key,
      },
    ];
  });
}

type Collected = {
  fps: number;
  bars: VideoBar[];
  transitions: TransitionMark[];
  marks: ActionMark[];
  cameraKeys: CameraKeyMark[];
};

function cameraKeyMarks(p: Placed): CameraKeyMark[] {
  const { item, from, to, key } = p;
  if (!item.nodeId || !item.camera) return [];
  const { nodeId } = item;
  return sortedCameraKeys(item.camera).flatMap((k, index) => {
    const frame = p.walk.inst.toAbs(item.from + k.frame, "start");
    if (frame < from - EPSILON || frame > to + EPSILON) return [];
    return [{ key: `${key}:camera:${index}`, itemId: item.id, nodeId, index, frame }];
  });
}

function visitTracks(acc: Collected, tracks: TimelineTrack[], walk: Walk): void {
  tracks.forEach((track, ti) => {
    let previousLength = Infinity;
    for (const item of track.items) {
      const p = place(item, walk, ti);
      const before = previousLength;
      previousLength = p.absEnd - p.absStart;
      if (p.to - p.from < EPSILON) continue;
      visitItem(acc, p, before);
    }
  });
}

function visitItem(acc: Collected, p: Placed, previousLength: number): void {
  const link = makeLink(p);
  const mark = transitionMark(p, previousLength);
  if (mark) acc.transitions.push(mark);
  acc.cameraKeys.push(...cameraKeyMarks(p));
  const group = groupOf(p.item);
  if (group) {
    visitGroup(acc, p, group, link);
    return;
  }
  acc.bars.push(leafBar(p, link, acc.bars.length));
  const recording = recordingOf(p.item);
  if (recording) acc.marks.push(...recordingMarks(p, recording, acc.fps));
}

function visitGroup(acc: Collected, p: Placed, group: GroupProps, link: ChainLink): void {
  const { item, walk, key } = p;
  const instances = groupInstances(item, group, walk.inst, p.absStart, p.absEnd);
  const looped = instances.length > 1;
  const chain = item.nodeId || item.handles || item.slot ? [link, ...walk.chain] : walk.chain;
  instances.forEach((inst, k) => {
    if (inst.hi - inst.lo < EPSILON) return;
    visitTracks(acc, group.tracks, {
      inst,
      path: looped ? `${key}#${k}` : key,
      chain,
      loop: looped ? k : walk.loop,
      timing: timingWith(walk.timing, nonEmptyRemap(group)),
    });
  });
}

function placeTransitions(transitions: TransitionMark[], bars: VideoBar[]): TransitionMark[] {
  return transitions.map((t) => {
    const lanes = new Set(bars.filter((b) => overlaps(b, t)).map((b) => b.lane));
    return { ...t, lanes: [...lanes].toSorted((a, b) => a - b) };
  });
}

export function flattenSpec(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
): FlatTimeline {
  const acc: Collected = { fps: spec.fps, bars: [], transitions: [], marks: [], cameraKeys: [] };
  const root: Instance = { toAbs: (c) => c, lo: 0, hi: spec.durationInFrames };
  visitTracks(acc, spec.tracks, { inst: root, path: "", chain: [], loop: 0, timing: "plain" });
  const { bars, lanes } = packLanes(acc.bars);
  const { audio, lanes: audioLanes } = packAudio(audioBars(spec, bars));
  return {
    fps: spec.fps,
    durationInFrames: spec.durationInFrames,
    bars,
    lanes,
    audio,
    audioLanes,
    transitions: placeTransitions(acc.transitions, bars),
    marks: acc.marks.toSorted((x, y) => x.frame - y.frame),
    cameraKeys: acc.cameraKeys,
  };
}

export function nodeSpans(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
): Record<string, FrameRange> {
  const spans: Record<string, FrameRange> = {};
  for (const bar of flattenSpec(spec).bars)
    for (const link of bar.chain) {
      const from = Math.round(link.from);
      const to = Math.round(link.to);
      if (!link.nodeId || to <= from) continue;
      const s = spans[link.nodeId];
      spans[link.nodeId] = s
        ? { from: Math.min(s.from, from), to: Math.max(s.to, to) }
        : { from, to };
    }
  return spans;
}
