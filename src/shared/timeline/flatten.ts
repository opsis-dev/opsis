import {
  groupOf,
  itemHasSound,
  lerpSegment,
  recordingOf,
  remapFrame,
  type GroupProps,
  type Remap,
} from "../compile/clip";
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
import { clamp, numberOr } from "../math";
import { isProjectRef } from "../asset-ref";
import { msToExactFrames } from "./time";

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
  remapped: boolean;
};

export type AudioBar = {
  key: string;
  id: string;
  nodeId?: string;
  channel: string;
  src: string;
  role: "music" | "effect" | "clip";
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
const EPS = 1e-6;

type Range = { from: number; to: number };

const overlaps = (a: Range, b: Range) => a.from < b.to - EPS && b.from < a.to - EPS;

function inverseRemap(remap: Remap, r: number, side: Side): number {
  if (remap.length === 0) return r;
  const [o0, i0] = remap[0];
  const [oN, iN] = remap[remap.length - 1];
  if (r < i0) {
    const next = remap.find(([, i]) => i > i0);
    const slope = next ? (next[0] - o0) / (next[1] - i0) : 1;
    return o0 - (i0 - r) * slope;
  }
  if (side === "start") {
    if (r <= i0) return o0;
    for (let k = 1; k < remap.length; k++) {
      const [a, ia] = remap[k - 1];
      const [b, ib] = remap[k];
      if (ib >= r) return ib === ia ? a : lerpSegment([ia, a], [ib, b], r);
    }
    return oN + (r - iN);
  }
  if (r >= iN) return oN + (r - iN);
  for (let k = remap.length - 1; k > 0; k--) {
    const [a, ia] = remap[k - 1];
    const [b, ib] = remap[k];
    if (ia <= r) return ib <= r || ib === ia ? b : lerpSegment([ia, a], [ib, b], r);
  }
  return o0;
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

function bisect(inst: Instance, a: number, b: number, abs: number): number {
  let lo = a;
  let hi = b;
  for (let i = 0; i < 24; i++) {
    const mid = (lo + hi) / 2;
    if (inst.toAbs(mid, "start") < abs) lo = mid;
    else hi = mid;
  }
  return (lo + hi) / 2;
}

function rateAt(inst: Instance, content: number): number {
  const d = inst.toAbs(content + 1, "start") - inst.toAbs(content, "start");
  return d > EPS ? 1 / d : 1;
}

type Walk = {
  inst: Instance;
  path: string;
  chain: ChainLink[];
  loop: number;
  remapped: boolean;
};

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
    end: to < p.absEnd - EPS ? contentAt(p, to) - item.from : item.durationInFrames,
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
    covers: enter.hold ? 0 : (to - from) / Math.max(EPS, shorter),
    ...(item.slot ? { slot: item.slot } : {}),
  };
}

function thumbnailOf(item: TimelineItem): BarThumbnail | null {
  const recording = recordingOf(item);
  if (recording) {
    const src = recording.proxySrc || recording.src;
    return src
      ? { kind: "filmstrip", src, speed: numberOr(recording.speed, 1) || 1, trimMs: 0 }
      : null;
  }
  const { src, trimBeforeMs } = item.props;
  if (typeof src !== "string" || !src) return null;
  switch (item.component) {
    case "core/video":
      return { kind: "filmstrip", src, speed: 1, trimMs: numberOr(trimBeforeMs, 0) };
    case "core/image":
    case "core/still":
      return isProjectRef(src) ? { kind: "still", src } : null;
    default:
      return null;
  }
}

function leafBar(p: Placed, link: ChainLink, order: number): VideoBar {
  const { item, walk, from, to } = p;
  const startContent = from > p.absStart + EPS ? contentAt(p, from) : item.from;
  const endContent = to < p.absEnd - EPS ? contentAt(p, to) : p.contentEnd;
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
    remapped: walk.remapped,
  };
}

function recordingMarks(p: Placed, props: RecordingProps, fps: number): ActionMark[] {
  const { item, from, to, key } = p;
  const speed = props.speed || 1;
  return props.log.actions.flatMap((action) => {
    if (action.kind === "wait" || action.kind === "focus") return [];
    const local = (action.t / 1000 / speed) * fps;
    const frame = p.walk.inst.toAbs(item.from + local, "start");
    if (frame < from || frame >= to) return [];
    return [
      {
        key: `${key}:${action.id}`,
        frame,
        kind: action.kind,
        label: actionLabel(action),
        ...(action.nodeId ? { nodeId: action.nodeId } : {}),
        barKey: key,
      },
    ];
  });
}

export function flattenSpec(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
): FlatTimeline {
  const bars: VideoBar[] = [];
  const transitions: TransitionMark[] = [];
  const marks: ActionMark[] = [];

  const visitGroup = (p: Placed, group: GroupProps, link: ChainLink) => {
    const { item, walk, key } = p;
    const instances = groupInstances(item, group, walk.inst, p.absStart, p.absEnd);
    const looped = instances.length > 1;
    const chain = item.nodeId || item.handles || item.slot ? [link, ...walk.chain] : walk.chain;
    instances.forEach((inst, k) => {
      if (inst.hi - inst.lo < EPS) return;
      visit(group.tracks, {
        inst,
        path: looped ? `${key}#${k}` : key,
        chain,
        loop: looped ? k : walk.loop,
        remapped: walk.remapped || !!group.remap?.length,
      });
    });
  };

  const visit = (tracks: TimelineTrack[], walk: Walk) => {
    tracks.forEach((track, ti) => {
      let previousLength = Infinity;
      for (const item of track.items) {
        const p = place(item, walk, ti);
        const before = previousLength;
        previousLength = p.absEnd - p.absStart;
        if (p.to - p.from < EPS) continue;
        const link = makeLink(p);
        const mark = transitionMark(p, before);
        if (mark) transitions.push(mark);
        const group = groupOf(item);
        if (group) {
          visitGroup(p, group, link);
          continue;
        }
        bars.push(leafBar(p, link, bars.length));
        const recording = recordingOf(item);
        if (recording) marks.push(...recordingMarks(p, recording, spec.fps));
      }
    });
  };

  const root: Instance = { toAbs: (c) => c, lo: 0, hi: spec.durationInFrames };
  visit(spec.tracks, { inst: root, path: "", chain: [], loop: 0, remapped: false });

  const lanes = packLanes(bars);
  for (const t of transitions) {
    const set = new Set<number>();
    for (const b of bars) if (overlaps(b, t)) set.add(b.lane);
    t.lanes = [...set].toSorted((a, b) => a - b);
  }
  const audio = audioBars(spec, bars);
  const audioLanes = packAudio(audio);
  return {
    fps: spec.fps,
    durationInFrames: spec.durationInFrames,
    bars,
    lanes,
    audio,
    audioLanes,
    transitions,
    marks: marks.toSorted((x, y) => x.frame - y.frame),
  };
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

const ROLE_ORDER = { music: 0, effect: 1, clip: 2 } as const;

function soundBar(a: AudioItem, spec: Pick<TimelineSpec, "durationInFrames" | "fps">): AudioBar[] {
  const from = Math.max(0, a.from);
  const to = Math.min(
    spec.durationInFrames,
    a.from + soundLength(a, spec.fps, spec.durationInFrames),
  );
  if (to <= from) return [];
  const bar: AudioBar = {
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
  return [bar];
}

function clipSoundBar(bar: VideoBar, fps: number): AudioBar[] {
  const { src, trimBeforeMs, loop, volume } = bar.props;
  if (!bar.hasSound || bar.remapped || typeof src !== "string" || !src) return [];
  return [
    {
      key: `clip:${bar.key}`,
      id: bar.key,
      nodeId: bar.nodeId,
      channel: bar.nodeId,
      src,
      role: "clip",
      from: bar.from,
      to: bar.to,
      lane: 0,
      trimBefore: Math.round(bar.contentStart + msToExactFrames(numberOr(trimBeforeMs, 0), fps)),
      loop: !!loop,
      volume: numberOr(volume, 1),
      bar,
    },
  ];
}

function audioBars(
  spec: Pick<TimelineSpec, "audio" | "durationInFrames" | "fps">,
  bars: VideoBar[],
): AudioBar[] {
  return [
    ...spec.audio.flatMap((a) => soundBar(a, spec)),
    ...bars.flatMap((bar) => clipSoundBar(bar, spec.fps)),
  ].toSorted((x, y) => ROLE_ORDER[x.role] - ROLE_ORDER[y.role] || x.from - y.from);
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
    while (lane < ends.length && ends[lane] > a.from + EPS) lane++;
    a.lane = lane;
    ends[lane] = a.to;
  }
  return ends.length;
}

const naming =
  (verb: string, alone = verb) =>
  (what: string) =>
    what ? `${verb} ${what}` : alone;

const ACTION_LABELS: Record<Action["kind"], (what: string) => string> = {
  click: naming("Click"),
  type: naming("Type"),
  hover: naming("Hover"),
  scroll: () => "Scroll",
  navigate: naming("Go to", "Go to page"),
  wait: (what) => what || "wait",
  chapter: (what) => what || "Chapter",
  key: naming("Press", "Press keys"),
  focus: (what) => what || "focus",
  drag: naming("Drag"),
};

function actionLabel(a: Action): string {
  return ACTION_LABELS[a.kind](a.label?.trim() || a.text?.trim() || "");
}

type HandleOwner = { link: ChainLink; index: number };

export function slotIndex(bar: Pick<VideoBar, "chain">): number {
  return bar.chain.findIndex((l) => !!l.slot);
}

export function findHandle(
  bar: Pick<VideoBar, "chain">,
  kind: EditHandleKind,
  order: "inner" | "outer" = "inner",
): HandleOwner | null {
  const limit = slotIndex(bar);
  const end = limit === -1 ? bar.chain.length : limit + 1;
  const range = Array.from({ length: end }, (_, i) => i);
  if (order === "outer") range.reverse();
  for (const i of range) {
    const link = bar.chain[i];
    if (link.handles?.[kind]) return { link, index: i };
  }
  return null;
}
