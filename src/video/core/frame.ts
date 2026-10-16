import { useVideoConfig } from "remotion";
import { clamp } from "../../shared/math";

export type Insets = { top: number; right: number; bottom: number; left: number };

const NONE: Insets = { top: 0, right: 0, bottom: 0, left: 0 };

export function shapeInsets(width: number, height: number): Insets {
  const aspect = width / height;
  if (aspect >= 1.3) return NONE;
  if (aspect >= 0.7)
    return { top: 0.03 * height, right: 0.03 * width, bottom: 0.05 * height, left: 0.03 * width };
  return { top: 0.1 * height, right: 0.05 * width, bottom: 0.16 * height, left: 0.05 * width };
}

export const isLandscape = (width: number, height: number): boolean => width / height >= 1.3;

export const overlayScale = (width: number, height: number): number =>
  Math.max(width, height) / 1920;

export function zoomBoost(sourceAspect: number, outAspect: number): number {
  if (!(sourceAspect > 0) || !(outAspect > 0)) return 1;
  return clamp(Math.sqrt(sourceAspect / outAspect), 1, 2);
}

export function useOverlayFrame(): { k: number; insets: Insets } {
  const { width, height } = useVideoConfig();
  return { k: overlayScale(width, height), insets: shapeInsets(width, height) };
}
