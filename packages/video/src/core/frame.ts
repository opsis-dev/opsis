import { useFilm } from "@opsis/runtime/clock.tsx";
import type React from "react";
import { designScaleFor } from "#video/camera/lib/design-width.ts";
import { clamp } from "#video/common/math.ts";
import { containBox, type Size } from "#video/core/still-layout.ts";
import type { ReframeMode } from "#video/generated/spec.ts";

type Insets = { top: number; right: number; bottom: number; left: number };

const NONE: Insets = { top: 0, right: 0, bottom: 0, left: 0 };

const LANDSCAPE_ASPECT = 1.3;
const SQUARISH_ASPECT = 0.7;
const SQUARISH_INSETS: Insets = { top: 0.03, right: 0.03, bottom: 0.05, left: 0.03 };
const TALL_INSETS: Insets = { top: 0.1, right: 0.05, bottom: 0.16, left: 0.05 };

export function isLandscape(width: number, height: number): boolean {
  return width / height >= LANDSCAPE_ASPECT;
}

const SAME_SHAPE_TOLERANCE = 0.02;

export function sameShape(a: Size, b: Size): boolean {
  return Math.abs(a.width / a.height - b.width / b.height) <= SAME_SHAPE_TOLERANCE;
}

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

export function verticalInsets({ top, bottom }: Insets): Insets {
  return { ...NONE, top, bottom };
}

export function overlayPadding(pad: number, insets: Insets): React.CSSProperties {
  return {
    paddingTop: pad + insets.top,
    paddingRight: pad + insets.right,
    paddingBottom: pad + insets.bottom,
    paddingLeft: pad + insets.left,
  };
}

export const JUSTIFY_AT = { top: "flex-start", center: "center", bottom: "flex-end" } as const;

function overlayScale(width: number, height: number): number {
  return designScaleFor(Math.max(width, height));
}

export function zoomBoost(sourceAspect: number, outAspect: number): number {
  return clamp(Math.sqrt(sourceAspect / outAspect), 1, 2);
}

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
  const { width, height } = useFilm();
  return { k: overlayScale(width, height), insets: shapeInsets(width, height) };
}
