import { msToExactFrames } from "@opsis/video/common/time";
import type { Marker, FlatTimeline } from "@/generated/bindings";

export const HEADER_W = 84;
export const RULER_H = 24;
export const MARKERS_H = 20;
export const VIDEO_H = 46;
export const AUDIO_H = 46;
export const BAR_INSET = 8;
export const SNAP_PX = 8;
const TAIL_PX = 240;
export const FIT_PADDING_PX = 32;
export const OFFSCREEN_PX = 64;
const VIDEO_TOP = RULER_H + MARKERS_H;

const EDGE_PX = 6;

export type Viewport = { left: number; width: number; pxPerFrame: number };

export type LaneLayout = { videoLanes: number; audioTop: number };

export function videoRowTop(layout: LaneLayout, lane: number): number {
  return VIDEO_TOP + (layout.videoLanes - 1 - lane) * VIDEO_H;
}

export function audioRowTop(layout: LaneLayout, lane: number): number {
  return layout.audioTop + lane * AUDIO_H;
}

export function barHeight(rowHeight: number): number {
  return rowHeight - BAR_INSET;
}

export function markerFrame(marker: Pick<Marker, "atMs">, fps: number): number {
  return msToExactFrames(marker.atMs, fps);
}

export type BarBox = { left: number; width: number };

const MIN_BAR_PX = 2;

export function barBox(from: number, to: number, ppf: number): BarBox {
  return {
    left: from * ppf,
    width: Math.max(MIN_BAR_PX, (to - from) * ppf),
  };
}

export function inView(view: Viewport, from: number, to: number): boolean {
  return (
    to * view.pxPerFrame >= view.left - OFFSCREEN_PX &&
    from * view.pxPerFrame <= view.left + view.width + OFFSCREEN_PX
  );
}

export function hitEdge(e: React.PointerEvent<HTMLElement>, width: number): "start" | "end" | null {
  const rect = e.currentTarget.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const edge = Math.min(EDGE_PX, width / 3);
  if (x <= edge) return "start";
  if (x >= rect.width - edge) return "end";
  return null;
}

function contentX(el: HTMLElement, clientX: number): number {
  return clientX - el.getBoundingClientRect().left + el.scrollLeft;
}

export function contentPoint(el: HTMLElement, clientX: number, clientY: number) {
  return {
    x: contentX(el, clientX),
    y: clientY - el.getBoundingClientRect().top + el.scrollTop,
    scrollTop: el.scrollTop,
  };
}

export function frameAtX(x: number, ppf: number): number {
  return (x - HEADER_W) / ppf;
}

export function clientXToFrame(el: HTMLElement, clientX: number, ppf: number): number {
  return frameAtX(contentX(el, clientX), ppf);
}

export function timelineGeometry(
  flat: Pick<FlatTimeline, "lanes" | "audioLanes" | "durationInFrames">,
  view: { left: number; width: number },
  ppf: number,
) {
  const videoLanes = Math.max(1, flat.lanes);
  const audioLanes = Math.max(1, flat.audioLanes);
  const audioTop = VIDEO_TOP + videoLanes * VIDEO_H;
  const viewport: Viewport = { left: view.left, width: view.width - HEADER_W, pxPerFrame: ppf };
  return {
    layout: { videoLanes, audioTop },
    audioLanes,
    totalH: audioTop + audioLanes * AUDIO_H,
    contentW: Math.max(view.width - HEADER_W, flat.durationInFrames * ppf + TAIL_PX),
    viewport,
  };
}
