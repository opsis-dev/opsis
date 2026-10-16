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
