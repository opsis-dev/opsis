import { useRef, useState } from "react";
import { round3, clamp } from "@/shared/math";
import type { Marker, Workflow } from "@/shared/schema/workflow";
import { setAudioVolume } from "@/shared/timeline/edits";
import type { AudioBar, FlatTimeline, VideoBar } from "@/shared/timeline/flatten";
import { snapFrame } from "@/shared/timeline/snap";
import { framesToMs } from "@/shared/timeline/time";
import { usePlayback } from "../../store/playback";
import { useUiStore } from "../../store/ui";
import { useWorkflowStore } from "../../store/workflow";
import { ENVELOPE_INFO, setEnvelopeKeys } from "./envelope";
import { scrubAt } from "./scrub";
import { moveMarker, reportEdit, selectNodes } from "./timeline-actions";
import {
  editContext,
  editOutcome,
  gestureSalt,
  isClick,
  marqueeIds,
  normalizeMarquee,
  snapTargetsFor,
  type AudioEditKind,
  type Drag,
  type EditDrag,
  type KeyDrag,
  type MarkerDrag,
  type MarqueeBox,
  type MarqueeDrag,
  type VideoEditKind,
  type VolumeDrag,
} from "./timeline-drag";
import { HEADER_W, RULER_H, SNAP_PX, type LaneLayout } from "./timeline-layout";
import { useTimeline } from "./timeline-state";
import type { TimelineZoom } from "./use-timeline-zoom";

type DragOptions = {
  scroller: React.RefObject<HTMLDivElement | null>;
  workflow: Workflow;
  flat: FlatTimeline;
  sourceNodeId: string | null;
  masterSrc: string | null;
  markers: Marker[];
  beats: number[];
  layout: LaneLayout;
  zoom: TimelineZoom;
};

export type KeyStart = Omit<KeyDrag, "kind" | "x0" | "y0" | "started">;

export type TimelineDrag = ReturnType<typeof useTimelineDrag>;

const VOLUME_RANGE = 2;

function passedThreshold(d: { started: boolean }, distance: number, min: number): boolean {
  if (d.started) return true;
  if (distance < min) return false;
  d.started = true;
  useWorkflowStore.getState().beginGesture();
  return true;
}

export function useTimelineDrag(opts: DragOptions) {
  const { scroller, workflow, flat, sourceNodeId, masterSrc, zoom } = opts;
  const { fps, durationInFrames: duration } = flat;
  const ppf = zoom.ppf;
  const snapping = useTimeline((s) => s.snapping);
  const drag = useRef<Drag | null>(null);
  const [guide, setGuide] = useState<number | null>(null);
  const [marquee, setMarquee] = useState<MarqueeBox | null>(null);

  const scrollerPoint = (clientX: number, clientY: number) => {
    const el = scroller.current;
    if (!el) return { x: 0, y: 0, scrollTop: 0 };
    const rect = el.getBoundingClientRect();
    return {
      x: clientX - rect.left + el.scrollLeft,
      y: clientY - rect.top + el.scrollTop,
      scrollTop: el.scrollTop,
    };
  };
  const xToFrame = (clientX: number) => (scrollerPoint(clientX, 0).x - HEADER_W) / ppf;
  const snapOn = (alt: boolean) => snapping && !alt;

  const snapTargets = (exclude: string | null) => {
    const { frame, inPoint, outPoint } = usePlayback.getState();
    return snapTargetsFor(
      { flat, markers: opts.markers, beats: opts.beats, fps, playhead: frame, inPoint, outPoint },
      exclude,
    );
  };

  const snapAt = (at: number, alt: boolean) =>
    snapOn(alt) ? snapFrame(at, snapTargets(null), SNAP_PX / ppf).frame : at;

  const capture = (e: React.PointerEvent) => scroller.current?.setPointerCapture(e.pointerId);

  const begin = (e: React.PointerEvent, d: Drag) => {
    capture(e);
    drag.current = d;
    zoom.hold();
  };

  const beginPointer = (e: React.PointerEvent<HTMLElement>): boolean => {
    if (e.button !== 0) return false;
    e.stopPropagation();
    e.currentTarget.closest<HTMLElement>("[data-timeline-root]")?.focus();
    capture(e);
    return true;
  };

  const editBase = (bar: VideoBar | AudioBar, clientX: number, ripple: boolean) => ({
    x0: clientX,
    base: workflow,
    flat,
    targets: snapTargets(bar.key),
    salt: gestureSalt(),
    started: false,
    last: null,
    ripple,
  });

  const startVideoEdit = (
    e: React.PointerEvent,
    kind: VideoEditKind,
    bar: VideoBar,
    ripple: boolean,
  ) => begin(e, { kind, bar, ...editBase(bar, e.clientX, ripple) });

  const startAudioEdit = (
    e: React.PointerEvent,
    kind: AudioEditKind,
    bar: AudioBar,
    ripple: boolean,
  ) => begin(e, { kind, bar, ...editBase(bar, e.clientX, ripple) });

  const startVolume = (e: React.PointerEvent, bar: AudioBar, height: number) =>
    begin(e, {
      kind: "volume",
      bar,
      y0: e.clientY,
      v0: bar.volume,
      base: workflow,
      started: false,
      height,
    });

  const startKey = (e: React.PointerEvent, key: KeyStart) => {
    e.stopPropagation();
    if (e.button !== 0) return;
    begin(e, { kind: "key", ...key, x0: e.clientX, y0: e.clientY, started: false });
  };

  const startMarker = (e: React.PointerEvent, marker: Marker) =>
    begin(e, { kind: "marker", id: marker.id, x0: e.clientX, at0: marker.atMs, started: false });

  const scrubTo = (f: number) => {
    usePlayback.getState().seek(f);
    scrubAt(masterSrc, f / fps);
  };

  const onRulerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    begin(e, { kind: "scrub" });
    scrubTo(Math.max(0, xToFrame(e.clientX)));
  };

  const onEmptyDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    const { x, y, scrollTop } = scrollerPoint(e.clientX, e.clientY);
    if (y < RULER_H + scrollTop && y >= scrollTop) return;
    begin(e, { kind: "marquee", x0: x, y0: y, x1: x, y1: y, additive: e.shiftKey });
  };

  const moveMarquee = (d: MarqueeDrag, e: React.PointerEvent) => {
    const { x, y } = scrollerPoint(e.clientX, e.clientY);
    d.x1 = x;
    d.y1 = y;
    setMarquee({ x0: d.x0, y0: d.y0, x1: x, y1: y });
  };

  const moveVolume = (d: VolumeDrag, e: React.PointerEvent) => {
    if (!passedThreshold(d, Math.abs(e.clientY - d.y0), 2)) return;
    const v = d.v0 - ((e.clientY - d.y0) / d.height) * VOLUME_RANGE;
    const r = setAudioVolume(editContext(d.base, fps, sourceNodeId, "v"), d.bar, v);
    if (r.ok) useWorkflowStore.getState().update(() => r.workflow);
  };

  const moveKey = (d: KeyDrag, e: React.PointerEvent) => {
    const distance = Math.abs(e.clientX - d.x0) + Math.abs(e.clientY - d.y0);
    if (!passedThreshold(d, distance, 3)) return;
    const { min, max } = ENVELOPE_INFO[d.prop];
    const key = d.keys[d.index];
    const frame = Math.max(0, Math.round(d.toLocal(xToFrame(e.clientX))));
    const dragged = ((e.clientY - d.y0) / Math.max(1, d.height - 10)) * (max - min);
    const value = round3(clamp(key.value - dragged, min, max));
    setEnvelopeKeys(
      d.nodeId,
      d.prop,
      d.keys.map((k, i) => (i === d.index ? { ...k, frame, value } : k)),
    );
  };

  const moveMarkerDrag = (d: MarkerDrag, e: React.PointerEvent) => {
    if (!passedThreshold(d, Math.abs(e.clientX - d.x0), 3)) return;
    let f = (d.at0 / 1000) * fps + (e.clientX - d.x0) / ppf;
    if (snapOn(e.altKey)) {
      const targets = snapTargets(null).filter((t) => t.kind !== "marker");
      f = snapFrame(f, targets, SNAP_PX / ppf).frame;
    }
    moveMarker(d.id, Math.max(0, framesToMs(f, fps)));
  };

  const moveEdit = (d: EditDrag, e: React.PointerEvent) => {
    if (!passedThreshold(d, Math.abs(e.clientX - d.x0), 3)) return;
    const ctx = editContext(d.base, fps, sourceNodeId, d.salt);
    const snap = {
      raw: (e.clientX - d.x0) / ppf,
      threshold: snapOn(e.altKey) ? SNAP_PX / ppf : 0,
    };
    const { result, snappedAt } = editOutcome(ctx, d, snap, e.shiftKey);
    setGuide(snappedAt);
    d.last = result;
    if (result.ok) useWorkflowStore.getState().update(() => result.workflow);
  };

  const onPointerMove = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    switch (d.kind) {
      case "scrub":
        return scrubTo(clamp(xToFrame(e.clientX), 0, Math.max(0, duration - 1)));
      case "marquee":
        return moveMarquee(d, e);
      case "volume":
        return moveVolume(d, e);
      case "key":
        return moveKey(d, e);
      case "marker":
        return moveMarkerDrag(d, e);
      default:
        return moveEdit(d, e);
    }
  };

  const finishMarquee = (d: MarqueeDrag) => {
    setMarquee(null);
    const box = normalizeMarquee(d);
    if (!isClick(box)) {
      selectNodes(marqueeIds(flat, box, opts.layout, ppf), d.additive, sourceNodeId);
      return;
    }
    if (!d.additive) useUiStore.getState().selectPinned([], sourceNodeId);
    usePlayback.getState().seek(Math.max(0, (d.x0 - HEADER_W) / ppf));
    useTimeline.getState().selectMarker(null);
  };

  const endGesture = (d: { started: boolean }) => {
    if (d.started) useWorkflowStore.getState().endGesture();
    return d.started;
  };

  const onPointerUp = (e: React.PointerEvent) => {
    const d = drag.current;
    drag.current = null;
    setGuide(null);
    zoom.release();
    if (scroller.current?.hasPointerCapture(e.pointerId))
      scroller.current.releasePointerCapture(e.pointerId);
    if (!d) return;
    switch (d.kind) {
      case "scrub":
        return;
      case "marquee":
        return finishMarquee(d);
      case "volume":
        endGesture(d);
        return;
      case "marker":
        if (!endGesture(d)) useTimeline.getState().selectMarker(d.id);
        return;
      case "key":
        if (!endGesture(d)) selectNodes([d.nodeId], false, sourceNodeId);
        return;
      default:
        if (endGesture(d)) reportEdit(d.last);
    }
  };

  return {
    guide,
    marquee,
    isDragging: () => drag.current !== null,
    xToFrame,
    snapAt,
    beginPointer,
    startVideoEdit,
    startAudioEdit,
    startVolume,
    startKey,
    startMarker,
    onRulerDown,
    onEmptyDown,
    onPointerMove,
    onPointerUp,
  };
}
