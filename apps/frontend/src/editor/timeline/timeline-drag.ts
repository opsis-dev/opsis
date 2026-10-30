import type {
  AudioBar,
  FlatTimeline,
  Keyframe,
  Marker,
  TimelineEdit,
  TimelineReply,
  VideoBar,
  EnvelopeKey,
} from "@/generated/bindings";
import { DRAG_START_PX } from "@/lib/dom";
import { snapFrame, snapSpan, type SnapTarget } from "@/timeline/snap";
import {
  AUDIO_H,
  HEADER_W,
  VIDEO_H,
  audioRowTop,
  markerFrame,
  videoRowTop,
  type LaneLayout,
} from "./timeline-layout";

type EditDragBase = {
  x0: number;
  targets: SnapTarget[];
  started: boolean;
  last: Promise<TimelineReply> | null;
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

type AudioEditKind = "audio-move" | "audio-trim-start" | "audio-trim-end";

export type EditTarget =
  | { domain: "video"; kind: VideoEditKind; bar: VideoBar }
  | { domain: "audio"; kind: AudioEditKind; bar: AudioBar };

export type EditDrag = EditDragBase & EditTarget;

export type VolumeDrag = {
  bar: AudioBar;
  y0: number;
  v0: number;
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
  scale: number;
  started: boolean;
};

export type KeyStart = Omit<KeyDrag, "x0" | "y0" | "started">;

export type MarkerDrag = { id: string; x0: number; from: number; started: boolean };

type SnapInput = { raw: number; threshold: number };

type EditOutcome = { edit: TimelineEdit; snappedAt: number | null };

type Snapped = { delta: number; snappedAt: number | null };

function settled(delta: number, target: SnapTarget | null): Snapped {
  return {
    delta: Math.round(delta),
    snappedAt: target?.frame ?? null,
  };
}

function snapMove(d: EditDrag, { raw, threshold }: SnapInput): Snapped {
  if (!threshold) return settled(raw, null);
  const s = snapSpan(d.bar.from, d.bar.to, raw, d.targets, threshold);
  return settled(s.delta, s.target);
}

function snapEdge(d: EditDrag, edge: number, { raw, threshold }: SnapInput): Snapped {
  if (!threshold) return settled(raw, null);
  const s = snapFrame(edge + raw, d.targets, threshold);
  return settled(s.frame - edge, s.target);
}

const EDGE_EDITS = {
  "trim-start": "trimStart",
  "roll-start": "rollStart",
  "roll-end": "rollEnd",
  speed: "speed",
} as const;

function videoOutcome(
  d: Extract<EditDrag, { domain: "video" }>,
  snap: SnapInput,
  shift: boolean,
): EditOutcome {
  const bar = d.bar.key;
  switch (d.kind) {
    case "move": {
      const { delta, snappedAt } = snapMove(d, snap);
      return { edit: { op: "move", bar, delta, ripple: d.ripple || shift }, snappedAt };
    }
    case "slip":
      return { edit: { op: "slip", bar, delta: Math.round(-snap.raw) }, snappedAt: null };
    case "trim-end": {
      const { delta, snappedAt } = snapEdge(d, d.bar.to, snap);
      return { edit: { op: "trimEnd", bar, delta, ripple: d.ripple }, snappedAt };
    }
    default: {
      const atEnd = d.kind === "roll-end" || d.kind === "speed";
      const { delta, snappedAt } = snapEdge(d, atEnd ? d.bar.to : d.bar.from, snap);
      return { edit: { op: EDGE_EDITS[d.kind], bar, delta }, snappedAt };
    }
  }
}

function audioOutcome(
  d: Extract<EditDrag, { domain: "audio" }>,
  snap: SnapInput,
  shift: boolean,
): EditOutcome {
  const bar = d.bar.key;
  if (d.kind === "audio-move") {
    const { delta, snappedAt } = snapMove(d, snap);
    return { edit: { op: "audioMove", bar, delta, ripple: d.ripple || shift }, snappedAt };
  }
  const atStart = d.kind === "audio-trim-start";
  const { delta, snappedAt } = snapEdge(d, atStart ? d.bar.from : d.bar.to, snap);
  return { edit: { op: atStart ? "audioTrimStart" : "audioTrimEnd", bar, delta }, snappedAt };
}

export function editOutcome(d: EditDrag, snap: SnapInput, shift: boolean): EditOutcome {
  return d.domain === "audio" ? audioOutcome(d, snap, shift) : videoOutcome(d, snap, shift);
}

type SnapSources = {
  flat: FlatTimeline;
  markers: Marker[];
  beats: number[];
  playhead: number;
  inPoint: number | null;
  outPoint: number | null;
};

export function snapTargetsFor(src: SnapSources, exclude: string | null): SnapTarget[] {
  const { flat } = src;
  const t: SnapTarget[] = [{ frame: src.playhead, kind: "playhead" }];
  for (const b of [...flat.bars, ...flat.audio])
    if (b.key !== exclude) t.push({ frame: b.from, kind: "edge" }, { frame: b.to, kind: "edge" });
  for (const m of src.markers) t.push({ frame: markerFrame(m, flat.fps), kind: "marker" });
  for (const m of flat.marks) t.push({ frame: m.frame, kind: "action" });
  for (const b of src.beats) t.push({ frame: b, kind: "beat" });
  if (src.inPoint !== null) t.push({ frame: src.inPoint, kind: "range" });
  if (src.outPoint !== null) t.push({ frame: src.outPoint, kind: "range" });
  t.push({ frame: 0, kind: "edge" }, { frame: flat.durationInFrames, kind: "edge" });
  return t;
}

export type MarqueeBox = { x0: number; x1: number; y0: number; y1: number };

export function isClick(box: MarqueeBox) {
  return box.x1 - box.x0 < DRAG_START_PX && box.y1 - box.y0 < DRAG_START_PX;
}

export function normalizeMarquee(d: MarqueeBox): MarqueeBox {
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
  function hits(from: number, to: number, top: number, height: number) {
    return from * ppf < box.x1 && to * ppf > box.x0 && top < box.y1 && top + height > box.y0;
  }
  for (const b of flat.bars)
    if (hits(b.from, b.to, videoRowTop(layout, b.lane), VIDEO_H)) ids.add(b.nodeId);
  for (const a of flat.audio)
    if (a.nodeId && hits(a.from, a.to, audioRowTop(layout, a.lane), AUDIO_H)) ids.add(a.nodeId);
  return [...ids];
}
