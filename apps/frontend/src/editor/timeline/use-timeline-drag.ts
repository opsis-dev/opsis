import { useState } from "react";
import type { Marker, AudioBar, CameraKeyMark } from "@/generated/bindings";
import { usePointerSession } from "@/lib/use-pointer-drag";
import { usePlayback } from "@/store/playback";
import { snapFrame } from "@/timeline/snap";
import { snapTargetsFor, type EditTarget, type KeyStart, type MarqueeBox } from "./timeline-drag";
import {
  cameraKeyGesture,
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
import { RULER_H, SNAP_PX, clientXToFrame, contentPoint } from "./timeline-layout";
import { useTimeline } from "./timeline-state";
import type { TimelineZoom } from "./use-timeline-zoom";

type DragOptions = Pick<GestureEnv, "flat" | "layout" | "sourceNodeId" | "masterSrc"> & {
  scroller: React.RefObject<HTMLDivElement | null>;
  markers: Marker[];
  beats: number[];
  zoom: TimelineZoom;
};

export type TimelineDrag = ReturnType<typeof useTimelineDrag>;

function inStickyRuler(y: number, scrollTop: number) {
  return y >= scrollTop && y < scrollTop + RULER_H;
}

function useTimelineSnap(
  { flat, markers, beats }: Pick<DragOptions, "flat" | "markers" | "beats">,
  ppf: number,
) {
  const snapping = useTimeline((s) => s.snapping);
  function snapOn(alt: boolean) {
    return snapping && !alt;
  }
  function snapTargets(exclude: string | null) {
    const { frame, inPoint, outPoint } = usePlayback.getState();
    return snapTargetsFor({ flat, markers, beats, playhead: frame, inPoint, outPoint }, exclude);
  }
  function snapAt(at: number, alt: boolean) {
    return snapOn(alt) ? snapFrame(at, snapTargets(null), SNAP_PX / ppf).frame : at;
  }
  return { snapOn, snapTargets, snapAt };
}

function useDragSession(scroller: DragOptions["scroller"], zoom: TimelineZoom, onEnd: () => void) {
  const session = usePointerSession<HTMLDivElement, ActiveDrag>({
    move: (e, active) => active.move(e),
    end: (active) => {
      zoom.release();
      onEnd();
      active.finish?.();
    },
  });
  function scrollerEl() {
    const el = scroller.current;
    if (!el) throw new Error("The timeline is not on screen.");
    return el;
  }
  function begin(e: React.PointerEvent, active: ActiveDrag) {
    scrollerEl().setPointerCapture(e.pointerId);
    session.track(active);
    zoom.hold();
  }
  return { scrollerEl, begin, isDragging: session.isActive, handlers: session.handlers };
}

export function useTimelineDrag(opts: DragOptions) {
  const { flat, sourceNodeId, masterSrc, layout, zoom } = opts;
  const ppf = zoom.ppf;
  const { snapOn, snapTargets, snapAt } = useTimelineSnap(opts, ppf);
  const [guide, setGuide] = useState<number | null>(null);
  const session = useDragSession(opts.scroller, zoom, () => setGuide(null));
  const { begin, scrollerEl } = session;
  const [marquee, setMarquee] = useState<MarqueeBox | null>(null);

  function point(clientX: number, clientY: number) {
    return contentPoint(scrollerEl(), clientX, clientY);
  }
  function xToFrame(clientX: number) {
    return clientXToFrame(scrollerEl(), clientX, ppf);
  }
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

  function beginPointer(e: React.PointerEvent<HTMLElement>): boolean {
    if (e.button !== 0) return false;
    e.stopPropagation();
    e.currentTarget.closest<HTMLElement>("[data-timeline-root]")?.focus();
    return true;
  }

  function startEdit(e: React.PointerEvent, target: EditTarget, ripple: boolean) {
    return begin(e, editGesture(env, e, target, ripple, setGuide));
  }

  function startVolume(e: React.PointerEvent, bar: AudioBar, height: number) {
    return begin(e, volumeGesture(e, bar, height));
  }

  function startKey(e: React.PointerEvent, key: KeyStart) {
    e.stopPropagation();
    if (e.button !== 0) return;
    begin(e, keyGesture(env, e, key));
  }

  function startCameraKey(e: React.PointerEvent, mark: CameraKeyMark) {
    e.stopPropagation();
    if (e.button !== 0) return;
    begin(e, cameraKeyGesture(env, e, mark));
  }

  function startMarker(e: React.PointerEvent, marker: Marker) {
    return begin(e, markerGesture(env, e, marker));
  }

  function onRulerDown(e: React.PointerEvent) {
    if (e.button !== 0) return;
    e.stopPropagation();
    begin(e, scrubGesture(env));
    scrubTo(env, xToFrame(e.clientX));
  }

  function onEmptyDown(e: React.PointerEvent) {
    if (e.button !== 0) return;
    const { x, y, scrollTop } = point(e.clientX, e.clientY);
    if (inStickyRuler(y, scrollTop)) return;
    begin(e, marqueeGesture(env, { x, y }, e.shiftKey, setMarquee));
  }

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
    startCameraKey,
    startMarker,
    onRulerDown,
    onEmptyDown,
    scrollerHandlers: session.handlers,
  };
}
