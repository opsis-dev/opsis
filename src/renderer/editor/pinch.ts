import { clamp } from "@/shared/math";
export const MIN_ZOOM = 0.1;
export const MAX_ZOOM = 2;
const PINCH_SPEED = 0.02;
const PINCH_STEP_LIMIT = 0.5;

type Viewport = { x: number; y: number; zoom: number };

export function pinchZoom(zoom: number, deltaY: number): number {
  const step = clamp(-deltaY * PINCH_SPEED, -PINCH_STEP_LIMIT, PINCH_STEP_LIMIT);
  return clamp(zoom * 2 ** step, MIN_ZOOM, MAX_ZOOM);
}

export function pinchViewport(view: Viewport, deltaY: number, px: number, py: number): Viewport {
  const zoom = pinchZoom(view.zoom, deltaY);
  return {
    x: px - ((px - view.x) * zoom) / view.zoom,
    y: py - ((py - view.y) * zoom) / view.zoom,
    zoom,
  };
}
