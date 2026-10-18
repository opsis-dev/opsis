import { useRef, useState } from "react";
import type { Marker } from "@/shared/schema/workflow";
import type { AudioBar } from "@/shared/timeline/flatten";
import { snapFrame } from "@/shared/timeline/snap";
import { usePlayback } from "../../store/playback";
import { snapTargetsFor, type EditTarget, type KeyStart, type MarqueeBox } from "./timeline-drag";
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
import { RULER_H, SNAP_PX, contentPoint, contentX, frameAtX } from "./timeline-layout";
import { useTimeline } from "./timeline-state";
import type { TimelineZoom } from "./use-timeline-zoom";

type DragOptions = Pick<
  GestureEnv,
  "workflow" | "flat" | "layout" | "sourceNodeId" | "masterSrc"
> & {
  scroller: React.RefObject<HTMLDivElement | null>;
  markers: Marker[];
  beats: number[];
  zoom: TimelineZoom;
};

export type TimelineDrag = ReturnType<typeof useTimelineDrag>;

function useTimelineSnap({ flat, markers, beats }: DragOptions, ppf: number) {
  const snapping = useTimeline((s) => s.snapping);
  const snapOn = (alt: boolean) => snapping && !alt;
  const snapTargets = (exclude: string | null) => {
    const { frame, inPoint, outPoint } = usePlayback.getState();
    return snapTargetsFor({ flat, markers, beats, playhead: frame, inPoint, outPoint }, exclude);
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
    workflow,
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
    return true;
  };

  const startEdit = (e: React.PointerEvent, target: EditTarget, ripple: boolean) =>
    begin(e, editGesture(env, e, target, ripple, setGuide));

  const startVolume = (e: React.PointerEvent, bar: AudioBar, height: number) =>
    begin(e, volumeGesture(env, e, bar, height));

  const startKey = (e: React.PointerEvent, key: KeyStart) => {
    e.stopPropagation();
    if (e.button !== 0) return;
    begin(e, keyGesture(env, e, key));
  };

  const startMarker = (e: React.PointerEvent, marker: Marker) =>
    begin(e, markerGesture(env, e, marker));

  const onRulerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    begin(e, scrubGesture(env));
    scrubTo(env, xToFrame(e.clientX));
  };

  const onEmptyDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    const { x, y, scrollTop } = point(e.clientX, e.clientY);
    if (y < RULER_H + scrollTop && y >= scrollTop) return;
    begin(e, marqueeGesture(env, { x, y }, e.shiftKey, setMarquee));
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
