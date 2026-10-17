import { useRef, useState } from "react";
import { round3, clamp } from "@/shared/math";
import { MAX_VOLUME } from "@/shared/nodes/params";
import type { Marker, Workflow } from "@/shared/schema/workflow";
import { setAudioVolume } from "@/shared/timeline/edits";
import type { AudioBar, FlatTimeline, VideoBar } from "@/shared/timeline/flatten";
import { snapFrame } from "@/shared/timeline/snap";
import { framesToMs, msToExactFrames } from "@/shared/timeline/time";
import { usePlayback } from "../../store/playback";
import { useUiStore } from "../../store/ui";
import { useWorkflowStore } from "../../store/workflow";
import { ENVELOPE_INFO, envelopeShift, setEnvelopeKeys } from "./envelope";
import { scrubAt } from "./scrub";
import { patchMarker, reportEdit, selectNodes } from "./timeline-actions";
import {
  editContext,
  editOutcome,
  gestureSalt,
  isClick,
  marqueeIds,
  normalizeMarquee,
  snapTargetsFor,
  type AudioEditKind,
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

export type KeyStart = Omit<KeyDrag, "x0" | "y0" | "started">;

type ActiveDrag = { move: (e: React.PointerEvent) => void; finish: () => void };

export type TimelineDrag = ReturnType<typeof useTimelineDrag>;

const VOLUME_DRAG_PX = 2;
const DRAG_START_PX = 3;

function beginGestureOnceMoved(d: { started: boolean }, distance: number, min: number): boolean {
  if (d.started) return true;
  if (distance < min) return false;
  d.started = true;
  useWorkflowStore.getState().beginGesture();
  return true;
}

function pointIn(el: HTMLDivElement | null, clientX: number, clientY: number) {
  if (!el) return { x: 0, y: 0, scrollTop: 0 };
  const rect = el.getBoundingClientRect();
  return {
    x: clientX - rect.left + el.scrollLeft,
    y: clientY - rect.top + el.scrollTop,
    scrollTop: el.scrollTop,
  };
}

function useTimelineSnap({ flat, markers, beats }: DragOptions, ppf: number) {
  const snapping = useTimeline((s) => s.snapping);
  const snapOn = (alt: boolean) => snapping && !alt;
  const snapTargets = (exclude: string | null) => {
    const { frame, inPoint, outPoint } = usePlayback.getState();
    return snapTargetsFor(
      { flat, markers, beats, fps: flat.fps, playhead: frame, inPoint, outPoint },
      exclude,
    );
  };
  const snapAt = (at: number, alt: boolean) =>
    snapOn(alt) ? snapFrame(at, snapTargets(null), SNAP_PX / ppf).frame : at;
  return { snapOn, snapTargets, snapAt };
}

export function useTimelineDrag(opts: DragOptions) {
  const { scroller, workflow, flat, sourceNodeId, masterSrc, zoom } = opts;
  const { fps, durationInFrames: duration } = flat;
  const ppf = zoom.ppf;
  const { snapOn, snapTargets, snapAt } = useTimelineSnap(opts, ppf);
  const drag = useRef<ActiveDrag | null>(null);
  const [guide, setGuide] = useState<number | null>(null);
  const [marquee, setMarquee] = useState<MarqueeBox | null>(null);

  const scrollerPoint = (clientX: number, clientY: number) =>
    pointIn(scroller.current, clientX, clientY);
  const xToFrame = (clientX: number) => (scrollerPoint(clientX, 0).x - HEADER_W) / ppf;

  const capture = (e: React.PointerEvent) => scroller.current?.setPointerCapture(e.pointerId);

  const begin = (e: React.PointerEvent, active: ActiveDrag) => {
    capture(e);
    drag.current = active;
    zoom.hold();
  };

  const endGesture = (d: { started: boolean }) => {
    if (d.started) useWorkflowStore.getState().endGesture();
    return d.started;
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

  const beginEdit = (e: React.PointerEvent, d: EditDrag) =>
    begin(e, {
      move: (ev) => moveEdit(d, ev),
      finish: () => {
        if (endGesture(d)) reportEdit(d.last);
      },
    });

  const startVideoEdit = (
    e: React.PointerEvent,
    kind: VideoEditKind,
    bar: VideoBar,
    ripple: boolean,
  ) => beginEdit(e, { kind, bar, ...editBase(bar, e.clientX, ripple) });

  const startAudioEdit = (
    e: React.PointerEvent,
    kind: AudioEditKind,
    bar: AudioBar,
    ripple: boolean,
  ) => beginEdit(e, { kind, bar, ...editBase(bar, e.clientX, ripple) });

  const startVolume = (e: React.PointerEvent, bar: AudioBar, height: number) => {
    const d: VolumeDrag = {
      bar,
      y0: e.clientY,
      v0: bar.volume,
      base: workflow,
      started: false,
      height,
    };
    begin(e, { move: (ev) => moveVolume(d, ev), finish: () => endGesture(d) });
  };

  const startKey = (e: React.PointerEvent, key: KeyStart) => {
    e.stopPropagation();
    if (e.button !== 0) return;
    const d: KeyDrag = { ...key, x0: e.clientX, y0: e.clientY, started: false };
    begin(e, {
      move: (ev) => moveKey(d, ev),
      finish: () => {
        if (!endGesture(d)) selectNodes([d.nodeId], false, sourceNodeId);
      },
    });
  };

  const startMarker = (e: React.PointerEvent, marker: Marker) => {
    const d: MarkerDrag = { id: marker.id, x0: e.clientX, at0: marker.atMs, started: false };
    begin(e, {
      move: (ev) => moveMarkerDrag(d, ev),
      finish: () => {
        if (!endGesture(d)) useTimeline.getState().selectMarker(d.id);
      },
    });
  };

  const scrubTo = (f: number) => {
    usePlayback.getState().seek(f);
    scrubAt(masterSrc, f / fps);
  };

  const onRulerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    begin(e, {
      move: (ev) => scrubTo(clamp(xToFrame(ev.clientX), 0, Math.max(0, duration - 1))),
      finish: () => {},
    });
    scrubTo(Math.max(0, xToFrame(e.clientX)));
  };

  const onEmptyDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    const { x, y, scrollTop } = scrollerPoint(e.clientX, e.clientY);
    if (y < RULER_H + scrollTop && y >= scrollTop) return;
    const d: MarqueeDrag = { x0: x, y0: y, x1: x, y1: y, additive: e.shiftKey };
    begin(e, { move: (ev) => moveMarquee(d, ev), finish: () => finishMarquee(d) });
  };

  const moveMarquee = (d: MarqueeDrag, e: React.PointerEvent) => {
    const { x, y } = scrollerPoint(e.clientX, e.clientY);
    d.x1 = x;
    d.y1 = y;
    setMarquee({ x0: d.x0, y0: d.y0, x1: x, y1: y });
  };

  const moveVolume = (d: VolumeDrag, e: React.PointerEvent) => {
    if (!beginGestureOnceMoved(d, Math.abs(e.clientY - d.y0), VOLUME_DRAG_PX)) return;
    const v = d.v0 - ((e.clientY - d.y0) / d.height) * MAX_VOLUME;
    const r = setAudioVolume(editContext(d.base, fps, sourceNodeId, "v"), d.bar, v);
    if (r.ok) useWorkflowStore.getState().update(() => r.workflow);
  };

  const moveKey = (d: KeyDrag, e: React.PointerEvent) => {
    const distance = Math.abs(e.clientX - d.x0) + Math.abs(e.clientY - d.y0);
    if (!beginGestureOnceMoved(d, distance, DRAG_START_PX)) return;
    const { min, max } = ENVELOPE_INFO[d.prop];
    const key = d.keys[d.index];
    const frame = Math.max(0, Math.round(d.toLocal(xToFrame(e.clientX))));
    const dragged = envelopeShift(d.prop, e.clientY - d.y0, d.height);
    const value = round3(clamp(key.value - dragged, min, max));
    setEnvelopeKeys(
      d.nodeId,
      d.prop,
      d.keys.map((k, i) => (i === d.index ? { ...k, frame, value } : k)),
    );
  };

  const moveMarkerDrag = (d: MarkerDrag, e: React.PointerEvent) => {
    if (!beginGestureOnceMoved(d, Math.abs(e.clientX - d.x0), DRAG_START_PX)) return;
    let f = msToExactFrames(d.at0, fps) + (e.clientX - d.x0) / ppf;
    if (snapOn(e.altKey)) {
      const targets = snapTargets(null).filter((t) => t.kind !== "marker");
      f = snapFrame(f, targets, SNAP_PX / ppf).frame;
    }
    patchMarker(d.id, { atMs: Math.max(0, framesToMs(f, fps)) });
  };

  const moveEdit = (d: EditDrag, e: React.PointerEvent) => {
    if (!beginGestureOnceMoved(d, Math.abs(e.clientX - d.x0), DRAG_START_PX)) return;
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

  const onPointerMove = (e: React.PointerEvent) => drag.current?.move(e);

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

  const onPointerUp = (e: React.PointerEvent) => {
    const d = drag.current;
    drag.current = null;
    setGuide(null);
    zoom.release();
    if (scroller.current?.hasPointerCapture(e.pointerId))
      scroller.current.releasePointerCapture(e.pointerId);
    d?.finish();
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
