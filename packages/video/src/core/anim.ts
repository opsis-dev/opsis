import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { spring } from "@opsis/runtime/spring.ts";
import type React from "react";

export const CLAMPED = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

export const CLAMP_RIGHT = { extrapolateRight: "clamp" } as const;

const FADED_OUT_OPACITY = 0.001;

export function isFadedOut(opacity: number): boolean {
  return opacity <= FADED_OUT_OPACITY;
}

export type Entrance = "rise" | "fade" | "blur" | "scale" | "none";

const ENTRANCE_SPRING = { damping: 200, mass: 0.8 };
export const ENTRANCE_FADE_FRAMES = 14;
const RISE_PX = 40;
const BLUR_PX = 18;
const SCALE_FROM = 0.86;
const SCALE_GROWTH = 0.14;

export function entranceStyle(
  entrance: Entrance,
  frame: number,
  fps: number,
  delay = 0,
): React.CSSProperties {
  if (entrance === "none") return {};
  const f = frame - delay;
  const s = spring({ frame: f, fps, config: ENTRANCE_SPRING });
  const opacity = interpolate(f, [0, ENTRANCE_FADE_FRAMES], [0, 1], CLAMPED);
  switch (entrance) {
    case "rise":
      return { opacity, transform: `translateY(${(1 - s) * RISE_PX}px)` };
    case "blur":
      return { opacity, filter: `blur(${(1 - s) * BLUR_PX}px)` };
    case "scale":
      return { opacity, transform: `scale(${SCALE_FROM + s * SCALE_GROWTH})` };
    case "fade":
      return { opacity };
  }
}

type Ease = (t: number) => number;

export function fadeWindow(
  t: number,
  fromT: number,
  toT: number,
  fade: { in: number; out: number; easeIn?: Ease; easeOut?: Ease },
): number {
  const fadeIn =
    fade.in > 0
      ? interpolate(t, [fromT, fromT + fade.in], [0, 1], { ...CLAMPED, easing: fade.easeIn })
      : 1;
  const fadeOut =
    fade.out > 0
      ? interpolate(t, [toT - fade.out, toT], [1, 0], { ...CLAMPED, easing: fade.easeOut })
      : 1;
  return Math.min(fadeIn, fadeOut);
}

const DRAW_ON_S = 0.45;

export function drawOn(t: number, fromT: number, length = DRAW_ON_S): number {
  return interpolate(t, [fromT, fromT + length], [0, 1], {
    ...CLAMPED,
    easing: Easing.inOut(Easing.cubic),
  });
}

export function zoomOver(frame: number, frames: number, to: number): number {
  return interpolate(frame, [0, frames], [1, to], CLAMP_RIGHT);
}

export function exitOpacity(frame: number, duration: number, frames = 10): number {
  return interpolate(frame, [duration - frames, duration], [1, 0], {
    ...CLAMPED,
    easing: Easing.in(Easing.quad),
  });
}
