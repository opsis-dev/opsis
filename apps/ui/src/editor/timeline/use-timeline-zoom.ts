import { useEffect, useState } from "react";
import { isPrimaryModifier } from "@/app/accelerators";
import { platform } from "@/lib/platform";
import { usePlayback } from "../../store/playback";
import { FIT_PADDING_PX, HEADER_W, clientXToFrame } from "./timeline-layout";
import { clampZoom, useTimeline } from "./timeline-state";

const WHEEL_ZOOM_STEP = 1.15;

const isPinchOrPrimary = (e: { ctrlKey: boolean; metaKey: boolean }): boolean =>
  e.ctrlKey || isPrimaryModifier(e, platform);

export type TimelineZoom = {
  ppf: number;
  zoomAround: (factor: number, anchorFrame?: number) => void;
  onWheel: (e: React.WheelEvent<HTMLElement>) => void;
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
      if (isPinchOrPrimary(e)) e.preventDefault();
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

  const onWheel = (e: React.WheelEvent<HTMLElement>) => {
    const el = e.currentTarget;
    if (isPinchOrPrimary(e)) {
      const factor = e.deltaY < 0 ? WHEEL_ZOOM_STEP : 1 / WHEEL_ZOOM_STEP;
      zoomAround(factor, clientXToFrame(el, e.clientX, ppf));
    } else if (
      !e.shiftKey &&
      Math.abs(e.deltaY) > Math.abs(e.deltaX) &&
      el.scrollHeight <= el.clientHeight
    ) {
      el.scrollLeft += e.deltaY;
    }
  };

  return {
    ppf,
    zoomAround,
    onWheel,
    hold: () => setHeld(ppf),
    release: () => setHeld(null),
  };
}
