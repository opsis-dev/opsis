import type { FlatTimeline } from "@/shared/timeline/flatten";

export const HEADER_W = 84;
export const RULER_H = 24;
export const MARKERS_H = 20;
export const VIDEO_H = 46;
export const AUDIO_H = 46;
export const BAR_INSET = 8;
export const SNAP_PX = 8;
export const TAIL_PX = 240;
export const FIT_PADDING_PX = 32;
export const OFFSCREEN_PX = 64;
export const VIDEO_TOP = RULER_H + MARKERS_H;

const EDGE_PX = 6;

export type Viewport = { left: number; width: number; pxPerFrame: number };

export type LaneLayout = { videoLanes: number; audioTop: number };

export const videoRowTop = (layout: LaneLayout, lane: number): number =>
  VIDEO_TOP + (layout.videoLanes - 1 - lane) * VIDEO_H;

export const audioRowTop = (layout: LaneLayout, lane: number): number =>
  layout.audioTop + lane * AUDIO_H;

export const barBox = (from: number, to: number, ppf: number) => ({
  left: from * ppf,
  width: Math.max(2, (to - from) * ppf),
});

export const inView = (view: Viewport, from: number, to: number): boolean =>
  to * view.pxPerFrame >= view.left - OFFSCREEN_PX &&
  from * view.pxPerFrame <= view.left + view.width + OFFSCREEN_PX;

export function hitEdge(e: React.PointerEvent<HTMLElement>, width: number): "start" | "end" | null {
  const rect = e.currentTarget.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const edge = Math.min(EDGE_PX, width / 3);
  if (x <= edge) return "start";
  if (x >= rect.width - edge) return "end";
  return null;
}

export const contentX = (el: HTMLElement, clientX: number): number =>
  clientX - el.getBoundingClientRect().left + el.scrollLeft;

export const contentPoint = (el: HTMLElement, clientX: number, clientY: number) => ({
  x: contentX(el, clientX),
  y: clientY - el.getBoundingClientRect().top + el.scrollTop,
  scrollTop: el.scrollTop,
});

export const frameAtX = (x: number, ppf: number): number => (x - HEADER_W) / ppf;

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
