import { remapFrame, type GroupProps } from "../compile/clip";
import { channelOf, soundLength } from "../compile/duck";
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

export type VideoBar = {
  key: string;
  itemId: string;
  nodeId: string;
  component: string;
  props: Record<string, unknown>;
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

export type ActionMark = {
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

export function inverseRemap(remap: Array<[number, number]>, r: number, side: Side): number {
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
      if (ib >= r) return ib === ia ? a : a + ((r - ia) / (ib - ia)) * (b - a);
    }
    return oN + (r - iN);
  }
  if (r >= iN) return oN + (r - iN);
  for (let k = remap.length - 1; k > 0; k--) {
    const [a, ia] = remap[k - 1];
    const [b, ib] = remap[k];
    if (ia <= r) return ib <= r ? b : ib === ia ? b : a + ((r - ia) / (ib - ia)) * (b - a);
  }
  return o0;
}

function groupInstances(item: TimelineItem, parent: Instance, absFrom: number, absTo: number) {
  const props = item.props as GroupProps;
  const startFrom = props.startFrom ?? 0;
  const remap = props.remap?.length ? props.remap : null;
  const loop = props.loopEvery && props.loopEvery > 0 ? props.loopEvery : 0;
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
  const count = Math.min(MAX_LOOPS, Math.max(1, Math.ceil(maxR / loop)));
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
  return d > 1e-6 ? 1 / d : 1;
}

export function flattenSpec(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
): FlatTimeline {
  const bars: VideoBar[] = [];
  const transitions: TransitionMark[] = [];
  const marks: ActionMark[] = [];
  const root: Instance = { toAbs: (c) => c, lo: 0, hi: spec.durationInFrames };

  const visit = (
    tracks: TimelineTrack[],
    inst: Instance,
    path: string,
    chain: ChainLink[],
    loop: number,
    remapped: boolean,
  ) => {
    tracks.forEach((track, ti) => {
      let prevLength = Infinity;
      for (const item of track.items) {
        const a = inst.toAbs(item.from, "start");
        const b = inst.toAbs(item.from + item.durationInFrames, "end");
        const length = b - a;
        const before = prevLength;
        prevLength = length;
        const from = Math.max(a, inst.lo);
        const to = Math.min(b, inst.hi);
        if (to - from < 1e-6) continue;
        const key = `${path}/${ti}:${item.id}`;
        const link: ChainLink = {
          itemId: item.id,
          ...(item.nodeId ? { nodeId: item.nodeId } : {}),
          ...(item.handles ? { handles: item.handles } : {}),
          ...(item.slot ? { slot: item.slot } : {}),
          rate: rateAt(inst, item.from),
          rateEnd: rateAt(inst, Math.max(item.from, item.from + item.durationInFrames - 1)),
          end:
            to < b - 1e-6
              ? bisect(inst, item.from, item.from + item.durationInFrames, to) - item.from
              : item.durationInFrames,
          from,
          to,
        };
        if (item.enter && item.enter.durationInFrames > 0) {
          const end = Math.min(to, inst.toAbs(item.from + item.enter.durationInFrames, "end"));
          transitions.push({
            key: `${key}:enter`,
            itemKey: key,
            lanes: [],
            from,
            to: end,
            kind: item.enter.kind,
            covers: item.enter.hold ? 0 : (end - from) / Math.max(1e-6, Math.min(length, before)),
            ...(item.slot ? { slot: item.slot } : {}),
          });
        }
        if (item.component === "core/group") {
          const props = item.props as GroupProps;
          const inner = props.tracks ?? [];
          const instances = groupInstances(item, inst, a, b);
          const nextChain = item.nodeId || item.handles || item.slot ? [link, ...chain] : chain;
          instances.forEach((child, k) => {
            if (child.hi - child.lo < 1e-6) return;
            visit(
              inner,
              child,
              instances.length > 1 ? `${key}#${k}` : key,
              nextChain,
              instances.length > 1 ? k : loop,
              remapped || !!props.remap?.length,
            );
          });
          continue;
        }
        const nodeId = item.nodeId ?? chain.find((l) => l.nodeId)?.nodeId ?? item.id;
        const startContent =
          from > a + 1e-6
            ? bisect(inst, item.from, item.from + item.durationInFrames, from)
            : item.from;
        const endContent =
          to < b - 1e-6
            ? bisect(inst, item.from, item.from + item.durationInFrames, to)
            : item.from + item.durationInFrames;
        const leafInst = inst;
        const bar: VideoBar = {
          key,
          itemId: item.id,
          nodeId,
          component: item.component,
          props: item.props,
          from,
          to,
          lane: 0,
          order: bars.length,
          chain: [link, ...chain],
          rate: rateAt(inst, startContent),
          contentStart: startContent - item.from,
          contentEnd: endContent - item.from,
          loop,
          localAt: (abs: number) =>
            bisect(leafInst, item.from, item.from + item.durationInFrames, abs) - item.from,
          hasSound: item.component === "core/video" && item.props.muted === false,
          remapped,
        };
        bars.push(bar);
        if (item.component === "core/recording") {
          const props = item.props as unknown as RecordingProps;
          const speed = props.speed || 1;
          for (const action of props.log?.actions ?? []) {
            if (action.kind === "wait" || action.kind === "focus") continue;
            const local = (action.t / 1000 / speed) * spec.fps;
            const frame = inst.toAbs(item.from + local, "start");
            if (frame < from || frame >= to) continue;
            marks.push({
              key: `${key}:${action.id}`,
              frame,
              kind: action.kind,
              label: actionLabel(action),
              ...(action.nodeId ? { nodeId: action.nodeId } : {}),
              barKey: key,
            });
          }
        }
      }
    });
  };

  visit(spec.tracks, root, "", [], 0, false);

  const lanes = packLanes(bars);
  for (const t of transitions) {
    const set = new Set<number>();
    for (const b of bars) if (b.from < t.to - 1e-6 && t.from < b.to - 1e-6) set.add(b.lane);
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
    for (let j = 0; j < i; j++) {
      const o = bars[j];
      if (o.from < bars[i].to - 1e-6 && bars[i].from < o.to - 1e-6) taken.add(o.lane);
    }
    let lane = 0;
    while (taken.has(lane)) lane++;
    bars[i].lane = lane;
    lanes = Math.max(lanes, lane + 1);
  }
  return lanes;
}

const ROLE_ORDER = { music: 0, effect: 1, clip: 2 } as const;

function audioBars(
  spec: Pick<TimelineSpec, "audio" | "durationInFrames" | "fps">,
  bars: VideoBar[],
): AudioBar[] {
  const out: AudioBar[] = [];
  for (const a of spec.audio ?? []) {
    const from = Math.max(0, a.from);
    const to = Math.min(
      spec.durationInFrames,
      a.from + soundLength(a, spec.fps, spec.durationInFrames),
    );
    if (to <= from) continue;
    out.push({
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
      volume: typeof a.volume === "number" ? a.volume : 1,
      ...(a.handles ? { handles: a.handles } : {}),
      item: a,
    });
  }
  for (const bar of bars) {
    if (!bar.hasSound || bar.remapped) continue;
    const src = String(bar.props.src ?? "");
    if (!src) continue;
    const trimBeforeMs = Number(bar.props.trimBeforeMs ?? 0);
    out.push({
      key: `clip:${bar.key}`,
      id: bar.key,
      nodeId: bar.nodeId,
      channel: bar.nodeId,
      src,
      role: "clip",
      from: bar.from,
      to: bar.to,
      lane: 0,
      trimBefore: Math.round(bar.contentStart + (trimBeforeMs / 1000) * spec.fps),
      loop: !!bar.props.loop,
      volume: typeof bar.props.volume === "number" ? bar.props.volume : 1,
      bar,
    });
  }
  return out.toSorted((x, y) => ROLE_ORDER[x.role] - ROLE_ORDER[y.role] || x.from - y.from);
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
    while (lane < ends.length && ends[lane] > a.from + 1e-6) lane++;
    a.lane = lane;
    ends[lane] = a.to;
  }
  return ends.length;
}

function actionLabel(a: Action): string {
  const what = a.label?.trim() || a.text?.trim() || "";
  switch (a.kind) {
    case "click":
      return what ? `Click ${what}` : "Click";
    case "type":
      return what ? `Type ${what}` : "Type";
    case "hover":
      return what ? `Hover ${what}` : "Hover";
    case "scroll":
      return "Scroll";
    case "navigate":
      return what ? `Go to ${what}` : "Go to page";
    case "key":
      return what ? `Press ${what}` : "Press keys";
    case "chapter":
      return what || "Chapter";
    case "drag":
      return what ? `Drag ${what}` : "Drag";
    default:
      return what || a.kind;
  }
}

export type HandleOwner = { link: ChainLink; index: number };

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
