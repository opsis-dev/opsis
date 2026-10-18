import { groupOf, itemHasSound, recordingOf, type GroupProps } from "../compile/clip";
import { lerpSegment, remapFrame, type Remap } from "../compile/remap";
import { channelOf, soundLength } from "../audio";
import type { RecordingProps } from "../compile/recording";
import type { Action } from "../schema/actions";
import type {
  AudioItem,
  EditHandleKind,
  EditHandles,
  Slot,
  TimelineItem,
  TimelineSpec,
  TimelineTrack,
} from "../schema/timeline";
import { clamp, EPSILON, numberOr } from "../math";
import { isProjectRef } from "../asset-ref";
import { msToExactFrames } from "./time";
import { CHANNEL_KINDS, type ChannelKind } from "../mix/types";

type Side = "start" | "end";

type Instance = {
  toAbs: (content: number, side: Side) => number;
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

export type FlatTimeline = {
  fps: number;
  durationInFrames: number;
  bars: VideoBar[];
  lanes: number;
  audio: AudioBar[];
  audioLanes: number;
  transitions: TransitionMark[];
  marks: ActionMark[];
};

const MAX_LOOPS = 64;

export type FrameRange = { from: number; to: number };

const overlaps = (a: FrameRange, b: FrameRange) =>
  a.from < b.to - EPSILON && b.from < a.to - EPSILON;

function beforeFirst(remap: Remap, r: number): number {
  const [o0, i0] = remap[0];
  const next = remap.find(([, i]) => i > i0);
  const slope = next ? (next[0] - o0) / (next[1] - i0) : 1;
  return o0 - (i0 - r) * slope;
}

const afterLast = ([oN, iN]: Remap[number], r: number): number => oN + (r - iN);

function earliestOut(remap: Remap, last: Remap[number], r: number): number {
  for (let k = 1; k < remap.length; k++) {
    const [a, ia] = remap[k - 1];
    const [b, ib] = remap[k];
    if (ib >= r) return ib === ia ? a : lerpSegment([ia, a], [ib, b], r);
  }
  return afterLast(last, r);
}

function latestOut(remap: Remap, r: number): number {
  for (let k = remap.length - 1; k > 0; k--) {
    const [a, ia] = remap[k - 1];
    const [b, ib] = remap[k];
    if (ia <= r) return ib <= r || ib === ia ? b : lerpSegment([ia, a], [ib, b], r);
  }
  return remap[0][0];
}

function inverseRemap(remap: Remap, r: number, side: Side): number {
  const last = remap.at(-1);
  if (!last) return r;
  const [o0, i0] = remap[0];
  if (r < i0) return beforeFirst(remap, r);
  if (side === "start") return r === i0 ? o0 : earliestOut(remap, last, r);
  return r >= last[1] ? afterLast(last, r) : latestOut(remap, r);
}

function groupInstances(
  item: TimelineItem,
  group: GroupProps,
  parent: Instance,
  absFrom: number,
  absTo: number,
): Instance[] {
  const startFrom = group.startFrom ?? 0;
  const remap = group.remap?.length ? group.remap : null;
  const loop = group.loopEvery && group.loopEvery > 0 ? group.loopEvery : 0;
  const dur = item.durationInFrames;
  const toOuter = (o: number, side: Side) => parent.toAbs(item.from + o, side);
  const inv = (r: number, side: Side) => (remap ? inverseRemap(remap, r, side) : r);
  const lo = Math.max(parent.lo, absFrom);
  const hi = Math.min(parent.hi, absTo);
  if (!loop) {
    return [
      {
        toAbs: (c: number, side: Side) => toOuter(inv(c - startFrom, side), side),
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
      toAbs: (c: number, side: Side) => toOuter(inv(c - startFrom + k * loop, side), side),
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

const recordingSpeed = (props: RecordingProps): number => props.speed || 1;

type MediaProps = { src: string; trimBeforeMs: number; loop: boolean; volume: number };

function mediaProps(props: Record<string, unknown>): MediaProps | null {
  const { src, trimBeforeMs, loop, volume } = props;
  if (typeof src !== "string" || !src) return null;
  return {
    src,
    trimBeforeMs: numberOr(trimBeforeMs, 0),
    loop: !!loop,
    volume: numberOr(volume, 1),
  };
}

function thumbnailOf(item: TimelineItem): BarThumbnail | null {
  const recording = recordingOf(item);
  if (recording) {
    const src = recording.proxySrc || recording.src;
    return src ? { kind: "filmstrip", src, speed: recordingSpeed(recording), trimMs: 0 } : null;
  }
  const media = mediaProps(item.props);
  if (!media) return null;
  switch (item.component) {
    case "core/video":
      return { kind: "filmstrip", src: media.src, speed: 1, trimMs: media.trimBeforeMs };
    case "core/image":
    case "core/still":
      return isProjectRef(media.src) ? { kind: "still", src: media.src } : null;
    default:
      return null;
  }
}

function leafBar(p: Placed, link: ChainLink, order: number): VideoBar {
  const { item, walk, from, to } = p;
  const startContent = from > p.absStart + EPSILON ? contentAt(p, from) : item.from;
  const endContent = to < p.absEnd - EPSILON ? contentAt(p, to) : p.contentEnd;
  return {
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
    hasSound: itemHasSound(item),
    sound: itemHasSound(item) ? soundWindow(p, from, to, startContent) : null,
  };
}

function recordingMarks(p: Placed, props: RecordingProps, fps: number): ActionMark[] {
  const { item, from, to, key } = p;
  const speed = recordingSpeed(props);
  return props.log.actions.flatMap((action) => {
    const label = ACTION_LABELS[action.kind];
    if (!label) return [];
    const local = (action.t / 1000 / speed) * fps;
    const frame = p.walk.inst.toAbs(item.from + local, "start");
    if (frame < from || frame >= to) return [];
    return [
      {
        key: `${key}:${action.id}`,
        frame,
        kind: action.kind,
        label: label(action.label?.trim() || action.text?.trim() || ""),
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
};

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
      timing: timingWith(walk.timing, group.remap?.length ? group.remap : null),
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
  const acc: Collected = { fps: spec.fps, bars: [], transitions: [], marks: [] };
  const root: Instance = { toAbs: (c) => c, lo: 0, hi: spec.durationInFrames };
  visitTracks(acc, spec.tracks, { inst: root, path: "", chain: [], loop: 0, timing: "plain" });
  const { bars } = acc;
  const lanes = packLanes(bars);
  const audio = audioBars(spec, bars);
  return {
    fps: spec.fps,
    durationInFrames: spec.durationInFrames,
    bars,
    lanes,
    audio,
    audioLanes: packAudio(audio),
    transitions: placeTransitions(acc.transitions, bars),
    marks: acc.marks.toSorted((x, y) => x.frame - y.frame),
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

function packLanes(bars: VideoBar[]): number {
  let lanes = 0;
  for (let i = 0; i < bars.length; i++) {
    const taken = new Set<number>();
    for (let j = 0; j < i; j++) if (overlaps(bars[j], bars[i])) taken.add(bars[j].lane);
    let lane = 0;
    while (taken.has(lane)) lane++;
    bars[i].lane = lane;
    lanes = Math.max(lanes, lane + 1);
  }
  return lanes;
}

const roleOrder = (role: ChannelKind): number => CHANNEL_KINDS.indexOf(role);

function soundBar(
  a: AudioItem,
  spec: Pick<TimelineSpec, "durationInFrames" | "fps">,
): AudioBar | null {
  const from = Math.max(0, a.from);
  const to = Math.min(
    spec.durationInFrames,
    a.from + soundLength(a, spec.fps, spec.durationInFrames),
  );
  if (to <= from) return null;
  return {
    key: `audio:${a.id}`,
    id: a.id,
    ...(a.nodeId ? { nodeId: a.nodeId } : {}),
    channel: channelOf(a),
    src: a.src,
    role: a.role ?? "effect",
    from,
    to,
    lane: 0,
    trimBefore: (a.trimBefore ?? 0) + Math.max(0, -a.from),
    loop: !!a.loop,
    volume: numberOr(a.volume, 1),
    ...(a.handles ? { handles: a.handles } : {}),
    item: a,
  };
}

function clipSoundBar(bar: VideoBar, fps: number): AudioBar | null {
  const media = mediaProps(bar.props);
  const sound = bar.sound;
  if (!sound || !media) return null;
  return {
    key: `clip:${bar.key}`,
    id: bar.key,
    nodeId: bar.nodeId,
    channel: bar.nodeId,
    src: media.src,
    role: "clip",
    from: sound.from,
    to: sound.to,
    lane: 0,
    trimBefore: Math.round(sound.contentStart + msToExactFrames(media.trimBeforeMs, fps)),
    loop: media.loop,
    volume: media.volume,
    bar,
  };
}

function audioBars(
  spec: Pick<TimelineSpec, "audio" | "durationInFrames" | "fps">,
  bars: VideoBar[],
): AudioBar[] {
  return [
    ...spec.audio.map((a) => soundBar(a, spec)),
    ...bars.map((bar) => clipSoundBar(bar, spec.fps)),
  ]
    .filter((a) => a !== null)
    .toSorted((x, y) => roleOrder(x.role) - roleOrder(y.role) || x.from - y.from);
}

function packAudio(audio: AudioBar[]): number {
  const ends: number[] = [];
  let lastRole: AudioBar["role"] | null = null;
  let base = 0;
  for (const a of audio) {
    if (a.role !== lastRole) {
      base = ends.length;
      lastRole = a.role;
    }
    let lane = base;
    while (lane < ends.length && ends[lane] > a.from + EPSILON) lane++;
    a.lane = lane;
    ends[lane] = a.to;
  }
  return ends.length;
}

const naming =
  (verb: string, alone = verb) =>
  (what: string) =>
    what ? `${verb} ${what}` : alone;

const ACTION_LABELS: Partial<Record<Action["kind"], (what: string) => string>> = {
  click: naming("Click"),
  type: naming("Type"),
  hover: naming("Hover"),
  scroll: () => "Scroll",
  navigate: naming("Go to", "Go to page"),
  chapter: (what) => what || "Chapter",
  key: naming("Press", "Press keys"),
  drag: naming("Drag"),
};

type HandleOwner = { link: ChainLink; index: number };

export function slotIndex(bar: Pick<VideoBar, "chain">): number {
  return bar.chain.findIndex((l) => !!l.slot);
}

type SlotLink = ChainLink & { slot: Slot };

export const slotLink = (bar: Pick<VideoBar, "chain">): SlotLink | undefined =>
  bar.chain.find((l): l is SlotLink => !!l.slot);

export function findHandle(
  bar: Pick<VideoBar, "chain">,
  kind: EditHandleKind,
  order: "inner" | "outer" = "inner",
): HandleOwner | null {
  const limit = slotIndex(bar);
  const end = limit === -1 ? bar.chain.length : limit + 1;
  const step = order === "outer" ? -1 : 1;
  for (let i = order === "outer" ? end - 1 : 0; i >= 0 && i < end; i += step) {
    const link = bar.chain[i];
    if (link.handles?.[kind]) return { link, index: i };
  }
  return null;
}
