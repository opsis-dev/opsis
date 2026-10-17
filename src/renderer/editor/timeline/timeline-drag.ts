import type { Keyframe } from "@/shared/schema/timeline";
import type { Marker, Workflow } from "@/shared/schema/workflow";
import { DRAG_START_PX } from "@/renderer/lib/dom";
import {
  HALF_FRAME,
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
import { msToExactFrames } from "@/shared/timeline/time";
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

type VideoEdit = { kind: VideoEditKind; bar: VideoBar };
type AudioEdit = { kind: AudioEditKind; bar: AudioBar };

export type EditTarget = VideoEdit | AudioEdit;

type VideoEditDrag = EditDragBase & VideoEdit;
type AudioEditDrag = EditDragBase & AudioEdit;

export type EditDrag = VideoEditDrag | AudioEditDrag;

const CUT_TOLERANCE_FRAMES = 1;

export type VolumeDrag = {
  bar: AudioBar;
  y0: number;
  v0: number;
  base: Workflow;
  started: boolean;
  height: number;
};

export type KeyDrag = {
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

export type MarkerDrag = { id: string; x0: number; at0: number; started: boolean };

export type MarqueeDrag = {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  additive: boolean;
};

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

type EditOutcome = { result: EditResult; snappedAt: number | null };

type Snapper = (raw: number, threshold: number) => { delta: number; target: SnapTarget | null };

function snapDelta({ raw, threshold }: SnapInput, snapper: Snapper) {
  const s = threshold ? snapper(raw, threshold) : { delta: raw, target: null };
  return { delta: Math.round(s.delta), snappedAt: s.target?.frame ?? null };
}

const snapMove = (d: EditDrag, snap: SnapInput) =>
  snapDelta(snap, (raw, threshold) => snapSpan(d.bar.from, d.bar.to, raw, d.targets, threshold));

const snapEdge = (d: EditDrag, edge: number, snap: SnapInput) =>
  snapDelta(snap, (raw, threshold) => {
    const s = snapFrame(edge + raw, d.targets, threshold);
    return { delta: s.frame - edge, target: s.target };
  });

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
  return { ...result, workflow: applyRipple(result.workflow, targets, delta, ctx.fps) };
}

function rippleMove(ctx: EditContext, d: EditDrag, result: EditResult, delta: number) {
  const key = moveHandleKey(d.bar);
  return rippleAfter(
    ctx,
    d,
    result,
    d.bar.from + HALF_FRAME,
    {
      container: d.kind === "move" ? containerOf(d.bar) : undefined,
      refs: new Set(key ? [key] : []),
    },
    delta,
  );
}

function neighbour(
  flat: FlatTimeline,
  bar: VideoBar,
  side: "left" | "right",
): VideoBar | undefined {
  const container = containerOf(bar);
  return flat.bars.find(
    (b) =>
      b.key !== bar.key &&
      containerOf(b) === container &&
      Math.abs(side === "right" ? b.from - bar.to : b.to - bar.from) < CUT_TOLERANCE_FRAMES,
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
  const edge = (at: number, edit: (delta: number) => EditResult): EditOutcome => {
    const { delta, snappedAt } = snapEdge(d, at, snap);
    return { result: edit(delta), snappedAt };
  };
  switch (d.kind) {
    case "move":
      return moved(ctx, d, snap, d.ripple || shift, (delta) => moveBar(ctx, d.bar, delta));
    case "slip":
      return { result: slipBar(ctx, d.bar, Math.round(-snap.raw)), snappedAt: null };
    case "trim-start":
      return edge(d.bar.from, (delta) => trimBar(ctx, d.bar, "start", delta));
    case "trim-end":
      return edge(d.bar.to, (delta) => {
        const trimmed = trimBar(ctx, d.bar, "end", delta);
        if (!d.ripple) return trimmed;
        const scope = { container: containerOf(d.bar), refs: new Set<string>() };
        return rippleAfter(ctx, d, trimmed, d.bar.to - HALF_FRAME, scope, delta);
      });
    case "roll-start":
      return edge(d.bar.from, (delta) => {
        const left = neighbour(d.flat, d.bar, "left");
        return left ? rollCut(ctx, left, d.bar, delta) : trimBar(ctx, d.bar, "start", delta);
      });
    case "roll-end":
      return edge(d.bar.to, (delta) => {
        const right = neighbour(d.flat, d.bar, "right");
        return right ? rollCut(ctx, d.bar, right, delta) : trimBar(ctx, d.bar, "end", delta);
      });
    case "speed":
      return edge(d.bar.to, (delta) =>
        speedBar(ctx, d.bar, Math.max(1, Math.round(d.bar.to - d.bar.from + delta))),
      );
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
  for (const m of src.markers) t.push({ frame: msToExactFrames(m.atMs, fps), kind: "marker" });
  for (const m of flat.marks) t.push({ frame: m.frame, kind: "action" });
  for (const b of src.beats) t.push({ frame: b, kind: "beat" });
  if (src.inPoint !== null) t.push({ frame: src.inPoint, kind: "range" });
  if (src.outPoint !== null) t.push({ frame: src.outPoint, kind: "range" });
  t.push({ frame: 0, kind: "edge" }, { frame: flat.durationInFrames, kind: "edge" });
  return t;
}

export type MarqueeBox = { x0: number; x1: number; y0: number; y1: number };

export const isClick = (box: MarqueeBox) =>
  box.x1 - box.x0 < DRAG_START_PX && box.y1 - box.y0 < DRAG_START_PX;

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
