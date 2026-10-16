import type { Keyframe } from "@/shared/schema/timeline";
import type { Marker, Workflow } from "@/shared/schema/workflow";
import {
  applyRipple,
  containerOf,
  moveAudio,
  moveBar,
  moveHandleKey,
  rippleTargets,
  rollCut,
  slipBar,
  speedBar,
  trimAudio,
  trimBar,
  type EditContext,
  type EditResult,
} from "@/shared/timeline/edits";
import type { AudioBar, FlatTimeline, VideoBar } from "@/shared/timeline/flatten";
import { snapFrame, snapSpan, type SnapTarget } from "@/shared/timeline/snap";
import type { EnvelopeKey } from "./envelope";
import {
  AUDIO_H,
  HEADER_W,
  VIDEO_H,
  audioRowTop,
  videoRowTop,
  type LaneLayout,
} from "./timeline-layout";

type EditDragBase = {
  x0: number;
  base: Workflow;
  flat: FlatTimeline;
  targets: SnapTarget[];
  salt: string;
  started: boolean;
  last: EditResult | null;
  ripple: boolean;
};

export type VideoEditKind =
  | "move"
  | "trim-start"
  | "trim-end"
  | "slip"
  | "speed"
  | "roll-start"
  | "roll-end";

export type AudioEditKind = "audio-move" | "audio-trim-start" | "audio-trim-end";

type VideoEditDrag = EditDragBase & { kind: VideoEditKind; bar: VideoBar };
type AudioEditDrag = EditDragBase & { kind: AudioEditKind; bar: AudioBar };

export type EditDrag = VideoEditDrag | AudioEditDrag;

export type VolumeDrag = {
  kind: "volume";
  bar: AudioBar;
  y0: number;
  v0: number;
  base: Workflow;
  started: boolean;
  height: number;
};

export type KeyDrag = {
  kind: "key";
  nodeId: string;
  prop: EnvelopeKey;
  index: number;
  keys: Keyframe[];
  toLocal: (abs: number) => number;
  x0: number;
  y0: number;
  height: number;
  started: boolean;
};

export type MarkerDrag = { kind: "marker"; id: string; x0: number; at0: number; started: boolean };

export type MarqueeDrag = {
  kind: "marquee";
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  additive: boolean;
};

export type Drag = EditDrag | VolumeDrag | KeyDrag | MarkerDrag | MarqueeDrag | { kind: "scrub" };

let gestureCounter = 0;
export const gestureSalt = () => (++gestureCounter).toString(36);

export function editContext(
  base: Workflow,
  fps: number,
  sourceNodeId: string | null,
  salt: string,
): EditContext {
  let n = 0;
  return {
    workflow: base,
    fps,
    sourceNodeId,
    makeId: (prefix) => `${prefix}_${salt}${(n++).toString(36)}`,
  };
}

type SnapInput = { raw: number; threshold: number };

export type EditOutcome = { result: EditResult; snappedAt: number | null };

function snapMove(d: EditDrag, { raw, threshold }: SnapInput) {
  const s = threshold
    ? snapSpan(d.bar.from, d.bar.to, raw, d.targets, threshold)
    : { delta: raw, target: null };
  return { delta: Math.round(s.delta), snappedAt: s.target?.frame ?? null };
}

function snapEdge(d: EditDrag, edge: number, { raw, threshold }: SnapInput) {
  const s = threshold
    ? snapFrame(edge + raw, d.targets, threshold)
    : { frame: edge + raw, target: null };
  return { delta: Math.round(s.frame - edge), snappedAt: s.target?.frame ?? null };
}

function rippleAfter(
  ctx: EditContext,
  d: EditDrag,
  result: EditResult,
  after: number,
  exclude: { container?: string; refs: Set<string> },
  delta: number,
): EditResult {
  if (!result.ok) return result;
  const targets = rippleTargets(ctx, d.flat.bars, d.flat.audio, after, exclude);
  return { ...result, workflow: applyRipple(ctx, result.workflow, targets, delta) };
}

function rippleMove(ctx: EditContext, d: EditDrag, result: EditResult, delta: number) {
  const key = moveHandleKey(d.bar);
  return rippleAfter(
    ctx,
    d,
    result,
    d.bar.from + 0.5,
    {
      container: d.kind === "move" ? containerOf(d.bar) : undefined,
      refs: new Set(key ? [key] : []),
    },
    delta,
  );
}

export function neighbour(
  flat: FlatTimeline,
  bar: VideoBar,
  side: "left" | "right",
): VideoBar | undefined {
  const container = containerOf(bar);
  return flat.bars.find(
    (b) =>
      b.key !== bar.key &&
      containerOf(b) === container &&
      (side === "right" ? Math.abs(b.from - bar.to) < 1 : Math.abs(b.to - bar.from) < 1),
  );
}

function moved(
  ctx: EditContext,
  d: EditDrag,
  snap: SnapInput,
  ripple: boolean,
  move: (delta: number) => EditResult,
): EditOutcome {
  const { delta, snappedAt } = snapMove(d, snap);
  const result = move(delta);
  return { result: ripple ? rippleMove(ctx, d, result, delta) : result, snappedAt };
}

function videoOutcome(
  ctx: EditContext,
  d: VideoEditDrag,
  snap: SnapInput,
  shift: boolean,
): EditOutcome {
  if (d.kind === "move")
    return moved(ctx, d, snap, d.ripple || shift, (delta) => moveBar(ctx, d.bar, delta));
  if (d.kind === "slip")
    return { result: slipBar(ctx, d.bar, Math.round(-snap.raw)), snappedAt: null };
  const atStart = d.kind === "trim-start" || d.kind === "roll-start";
  const { delta, snappedAt } = snapEdge(d, atStart ? d.bar.from : d.bar.to, snap);
  const done = (result: EditResult) => ({ result, snappedAt });
  switch (d.kind) {
    case "trim-start":
      return done(trimBar(ctx, d.bar, "start", delta));
    case "trim-end": {
      const trimmed = trimBar(ctx, d.bar, "end", delta);
      if (!d.ripple) return done(trimmed);
      const scope = { container: containerOf(d.bar), refs: new Set<string>() };
      return done(rippleAfter(ctx, d, trimmed, d.bar.to - 0.5, scope, delta));
    }
    case "roll-start": {
      const left = neighbour(d.flat, d.bar, "left");
      return done(left ? rollCut(ctx, left, d.bar, delta) : trimBar(ctx, d.bar, "start", delta));
    }
    case "roll-end": {
      const right = neighbour(d.flat, d.bar, "right");
      return done(right ? rollCut(ctx, d.bar, right, delta) : trimBar(ctx, d.bar, "end", delta));
    }
    default:
      return done(speedBar(ctx, d.bar, Math.max(1, Math.round(d.bar.to - d.bar.from + delta))));
  }
}

function audioOutcome(
  ctx: EditContext,
  d: AudioEditDrag,
  snap: SnapInput,
  shift: boolean,
): EditOutcome {
  if (d.kind === "audio-move")
    return moved(ctx, d, snap, d.ripple || shift, (delta) => moveAudio(ctx, d.bar, delta));
  const side = d.kind === "audio-trim-start" ? "start" : "end";
  const { delta, snappedAt } = snapEdge(d, side === "start" ? d.bar.from : d.bar.to, snap);
  return { result: trimAudio(ctx, d.bar, side, delta), snappedAt };
}

const isAudioDrag = (d: EditDrag): d is AudioEditDrag => d.kind.startsWith("audio-");

export function editOutcome(
  ctx: EditContext,
  d: EditDrag,
  snap: SnapInput,
  shift: boolean,
): EditOutcome {
  return isAudioDrag(d) ? audioOutcome(ctx, d, snap, shift) : videoOutcome(ctx, d, snap, shift);
}

type SnapSources = {
  flat: FlatTimeline;
  markers: Marker[];
  beats: number[];
  fps: number;
  playhead: number;
  inPoint: number | null;
  outPoint: number | null;
};

export function snapTargetsFor(src: SnapSources, exclude: string | null): SnapTarget[] {
  const { flat, fps } = src;
  const t: SnapTarget[] = [{ frame: src.playhead, kind: "playhead" }];
  for (const b of [...flat.bars, ...flat.audio])
    if (b.key !== exclude) t.push({ frame: b.from, kind: "edge" }, { frame: b.to, kind: "edge" });
  for (const m of src.markers) t.push({ frame: (m.atMs / 1000) * fps, kind: "marker" });
  for (const m of flat.marks) t.push({ frame: m.frame, kind: "action" });
  for (const b of src.beats) t.push({ frame: b, kind: "beat" });
  if (src.inPoint !== null) t.push({ frame: src.inPoint, kind: "range" });
  if (src.outPoint !== null) t.push({ frame: src.outPoint, kind: "range" });
  t.push({ frame: 0, kind: "edge" }, { frame: flat.durationInFrames, kind: "edge" });
  return t;
}

export type MarqueeBox = { x0: number; x1: number; y0: number; y1: number };

export const isClick = (box: MarqueeBox) => box.x1 - box.x0 < 3 && box.y1 - box.y0 < 3;

export function normalizeMarquee(d: MarqueeDrag): MarqueeBox {
  return {
    x0: Math.min(d.x0, d.x1) - HEADER_W,
    x1: Math.max(d.x0, d.x1) - HEADER_W,
    y0: Math.min(d.y0, d.y1),
    y1: Math.max(d.y0, d.y1),
  };
}

export function marqueeIds(
  flat: FlatTimeline,
  box: MarqueeBox,
  layout: LaneLayout,
  ppf: number,
): string[] {
  const ids = new Set<string>();
  const hits = (from: number, to: number, top: number, height: number) =>
    from * ppf < box.x1 && to * ppf > box.x0 && top < box.y1 && top + height > box.y0;
  for (const b of flat.bars)
    if (hits(b.from, b.to, videoRowTop(layout, b.lane), VIDEO_H)) ids.add(b.nodeId);
  for (const a of flat.audio)
    if (a.nodeId && hits(a.from, a.to, audioRowTop(layout, a.lane), AUDIO_H)) ids.add(a.nodeId);
  return [...ids];
}
