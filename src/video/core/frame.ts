import { useVideoConfig } from "remotion";
import { clamp } from "../../shared/math";
import type { ReframeMode } from "../../shared/schema/timeline";
import { containBox, type Size } from "./still-layout";
import { designScaleFor } from "../camera/lib/design-width";

type Insets = { top: number; right: number; bottom: number; left: number };

const NONE: Insets = { top: 0, right: 0, bottom: 0, left: 0 };

const LANDSCAPE_ASPECT = 1.3;

export const isLandscape = (width: number, height: number): boolean =>
  width / height >= LANDSCAPE_ASPECT;

export function shapeInsets(width: number, height: number): Insets {
  if (isLandscape(width, height)) return NONE;
  const aspect = width / height;
  if (aspect >= 0.7)
    return { top: 0.03 * height, right: 0.03 * width, bottom: 0.05 * height, left: 0.03 * width };
  return { top: 0.1 * height, right: 0.05 * width, bottom: 0.16 * height, left: 0.05 * width };
}

export const overlayScale = (width: number, height: number): number =>
  designScaleFor(Math.max(width, height));

export const zoomBoost = (sourceAspect: number, outAspect: number): number =>
  clamp(Math.sqrt(sourceAspect / outAspect), 1, 2);

export type Reframe =
  | { mode: "same" }
  | { mode: "contain"; box: Size; narrow?: number }
  | { mode: "fit"; height: number }
  | { mode: "cover"; box: Size; follow: boolean };

const SAME_SHAPE_TOLERANCE = 0.02;

export function reframeLayout(
  source: Size,
  canvas: Size,
  reframe: ReframeMode,
  fullPage: boolean,
): Reframe {
  const sourceAspect = source.width / source.height;
  const outAspect = canvas.width / canvas.height;
  if (Math.abs(sourceAspect - outAspect) <= SAME_SHAPE_TOLERANCE) return { mode: "same" };
  const narrower = outAspect < sourceAspect;
  if (reframe === "follow" && (narrower || fullPage))
    return {
      mode: "contain",
      box: containBox(canvas, sourceAspect),
      ...(narrower ? { narrow: zoomBoost(sourceAspect, outAspect) } : {}),
    };
  if (reframe === "fit") return { mode: "fit", height: canvas.width / sourceAspect };
  const width = Math.max(canvas.width, canvas.height * sourceAspect);
  return {
    mode: "cover",
    box: { width, height: width / sourceAspect },
    follow: reframe === "follow",
  };
}

export function useOverlayFrame(): { k: number; insets: Insets } {
  const { width, height } = useVideoConfig();
  return { k: overlayScale(width, height), insets: shapeInsets(width, height) };
}
