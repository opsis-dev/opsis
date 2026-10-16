import { useEffect, useState } from "react";
import { usePlayback } from "../../store/playback";
import { FIT_PADDING_PX, HEADER_W } from "./timeline-layout";
import { clampZoom, useTimeline } from "./timeline-state";

export type TimelineZoom = {
  ppf: number;
  zoomAround: (factor: number, anchorFrame?: number) => void;
  hold: () => void;
  release: () => void;
};

export function useTimelineZoom(
  scroller: React.RefObject<HTMLDivElement | null>,
  viewWidth: number,
  duration: number,
): TimelineZoom {
  const zoom = useTimeline((s) => s.pxPerFrame);
  const fit = useTimeline((s) => s.fit);
  const [held, setHeld] = useState<number | null>(null);
  const fitZoom = clampZoom((viewWidth - HEADER_W - FIT_PADDING_PX) / Math.max(1, duration));
  const ppf = fit ? (held ?? fitZoom) : zoom;

  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const block = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) e.preventDefault();
    };
    el.addEventListener("wheel", block, { passive: false });
    return () => el.removeEventListener("wheel", block);
  }, [scroller]);

  const zoomAround = (factor: number, anchorFrame?: number) => {
    const el = scroller.current;
    if (!el) return;
    const anchor = anchorFrame ?? usePlayback.getState().frame;
    const screenX = anchor * ppf - el.scrollLeft;
    const next = clampZoom(ppf * factor);
    useTimeline.getState().setZoom(next);
    requestAnimationFrame(() => {
      el.scrollLeft = Math.max(0, anchor * next - screenX);
    });
  };

  return {
    ppf,
    zoomAround,
    hold: () => setHeld(ppf),
    release: () => setHeld(null),
  };
}
