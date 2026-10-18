import type React from "react";
import { useVideoConfig } from "remotion";
import { clamp } from "../../shared/math";
import type { ReframeMode } from "../../shared/schema/timeline";
import { designScaleFor } from "../camera/lib/design-width";
import { containBox, type Size } from "./still-layout";

type Insets = { top: number; right: number; bottom: number; left: number };

const NONE: Insets = { top: 0, right: 0, bottom: 0, left: 0 };

const LANDSCAPE_ASPECT = 1.3;
const SQUARISH_ASPECT = 0.7;
const SQUARISH_INSETS: Insets = { top: 0.03, right: 0.03, bottom: 0.05, left: 0.03 };
const TALL_INSETS: Insets = { top: 0.1, right: 0.05, bottom: 0.16, left: 0.05 };

export const isLandscape = (width: number, height: number): boolean =>
  width / height >= LANDSCAPE_ASPECT;

const SAME_SHAPE_TOLERANCE = 0.02;

export const sameShape = (a: Size, b: Size): boolean =>
  Math.abs(a.width / a.height - b.width / b.height) <= SAME_SHAPE_TOLERANCE;

export function shapeInsets(width: number, height: number): Insets {
  if (isLandscape(width, height)) return NONE;
  const f = width / height >= SQUARISH_ASPECT ? SQUARISH_INSETS : TALL_INSETS;
  return {
    top: f.top * height,
    right: f.right * width,
    bottom: f.bottom * height,
    left: f.left * width,
  };
}

export const verticalInsets = ({ top, bottom }: Insets): Insets => ({ ...NONE, top, bottom });

export const overlayPadding = (pad: number, insets: Insets): React.CSSProperties => ({
  paddingTop: pad + insets.top,
  paddingRight: pad + insets.right,
  paddingBottom: pad + insets.bottom,
  paddingLeft: pad + insets.left,
});

export const JUSTIFY_AT = { top: "flex-start", center: "center", bottom: "flex-end" } as const;

const overlayScale = (width: number, height: number): number =>
  designScaleFor(Math.max(width, height));

export const zoomBoost = (sourceAspect: number, outAspect: number): number =>
  clamp(Math.sqrt(sourceAspect / outAspect), 1, 2);

export type Reframe =
  | { mode: "same" }
  | { mode: "contain"; box: Size; narrow?: number }
  | { mode: "fit"; height: number }
  | { mode: "cover"; box: Size; follow: boolean };

export function reframeLayout(
  source: Size,
  canvas: Size,
  reframe: ReframeMode,
  fullPage: boolean,
): Reframe {
  if (sameShape(source, canvas)) return { mode: "same" };
  const sourceAspect = source.width / source.height;
  const outAspect = canvas.width / canvas.height;
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
