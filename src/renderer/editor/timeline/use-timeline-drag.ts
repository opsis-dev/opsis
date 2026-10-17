import { useRef, useState } from "react";
import type { Marker, Workflow } from "@/shared/schema/workflow";
import type { AudioBar, FlatTimeline } from "@/shared/timeline/flatten";
import { snapFrame } from "@/shared/timeline/snap";
import { usePlayback } from "../../store/playback";
import {
  gestureSalt,
  snapTargetsFor,
  type EditTarget,
  type KeyDrag,
  type MarqueeBox,
} from "./timeline-drag";
import {
  editGesture,
  keyGesture,
  markerGesture,
  marqueeGesture,
  scrubGesture,
  scrubTo,
  volumeGesture,
  type ActiveDrag,
  type GestureEnv,
} from "./timeline-gestures";
import {
  RULER_H,
  SNAP_PX,
  contentPoint,
  contentX,
  frameAtX,
  type LaneLayout,
} from "./timeline-layout";
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

export type TimelineDrag = ReturnType<typeof useTimelineDrag>;

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

function useDragSession(scroller: DragOptions["scroller"], zoom: TimelineZoom) {
  const drag = useRef<ActiveDrag | null>(null);
  const scrollerEl = () => {
    const el = scroller.current;
    if (!el) throw new Error("The timeline is not on screen.");
    return el;
  };
  const begin = (e: React.PointerEvent, active: ActiveDrag) => {
    scrollerEl().setPointerCapture(e.pointerId);
    drag.current = active;
    zoom.hold();
  };
  const end = (e: React.PointerEvent<HTMLElement>) => {
    const active = drag.current;
    drag.current = null;
    zoom.release();
    const el = e.currentTarget;
    if (el.hasPointerCapture(e.pointerId)) el.releasePointerCapture(e.pointerId);
    active?.finish?.();
  };
  return {
    scrollerEl,
    begin,
    end,
    move: (e: React.PointerEvent) => drag.current?.move(e),
    isDragging: () => drag.current !== null,
  };
}

export function useTimelineDrag(opts: DragOptions) {
  const { workflow, flat, sourceNodeId, masterSrc, layout, zoom } = opts;
  const ppf = zoom.ppf;
  const { snapOn, snapTargets, snapAt } = useTimelineSnap(opts, ppf);
  const session = useDragSession(opts.scroller, zoom);
  const { begin, scrollerEl } = session;
  const [guide, setGuide] = useState<number | null>(null);
  const [marquee, setMarquee] = useState<MarqueeBox | null>(null);

  const point = (clientX: number, clientY: number) => contentPoint(scrollerEl(), clientX, clientY);
  const xToFrame = (clientX: number) => frameAtX(contentX(scrollerEl(), clientX), ppf);
  const env: GestureEnv = {
    flat,
    layout,
    ppf,
    sourceNodeId,
    masterSrc,
    xToFrame,
    point,
    snapOn,
    snapTargets,
  };

  const beginPointer = (e: React.PointerEvent<HTMLElement>): boolean => {
    if (e.button !== 0) return false;
    e.stopPropagation();
    e.currentTarget.closest<HTMLElement>("[data-timeline-root]")?.focus();
    scrollerEl().setPointerCapture(e.pointerId);
    return true;
  };

  const startEdit = (e: React.PointerEvent, target: EditTarget, ripple: boolean) =>
    begin(
      e,
      editGesture(
        env,
        {
          ...target,
          x0: e.clientX,
          base: workflow,
          flat,
          targets: snapTargets(target.bar.key),
          salt: gestureSalt(),
          started: false,
          last: null,
          ripple,
        },
        setGuide,
      ),
    );

  const startVolume = (e: React.PointerEvent, bar: AudioBar, height: number) =>
    begin(
      e,
      volumeGesture(env, {
        bar,
        y0: e.clientY,
        v0: bar.volume,
        base: workflow,
        started: false,
        height,
      }),
    );

  const startKey = (e: React.PointerEvent, key: KeyStart) => {
    e.stopPropagation();
    if (e.button !== 0) return;
    begin(e, keyGesture(env, { ...key, x0: e.clientX, y0: e.clientY, started: false }));
  };

  const startMarker = (e: React.PointerEvent, marker: Marker) =>
    begin(
      e,
      markerGesture(env, { id: marker.id, x0: e.clientX, at0: marker.atMs, started: false }),
    );

  const onRulerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    begin(e, scrubGesture(env));
    scrubTo(env, Math.max(0, xToFrame(e.clientX)));
  };

  const onEmptyDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    const { x, y, scrollTop } = point(e.clientX, e.clientY);
    if (y < RULER_H + scrollTop && y >= scrollTop) return;
    const d = { x0: x, y0: y, x1: x, y1: y, additive: e.shiftKey };
    begin(e, marqueeGesture(env, d, setMarquee));
  };

  const onPointerUp = (e: React.PointerEvent<HTMLElement>) => {
    setGuide(null);
    session.end(e);
  };

  return {
    guide,
    marquee,
    isDragging: session.isDragging,
    xToFrame,
    snapAt,
    beginPointer,
    startEdit,
    startVolume,
    startKey,
    startMarker,
    onRulerDown,
    onEmptyDown,
    onPointerMove: session.move,
    onPointerUp,
  };
}
