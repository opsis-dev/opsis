import type React from "react";
import { Easing, interpolate, spring } from "remotion";

export const CLAMPED = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

export const visibleBackground = (background?: string): string | undefined =>
  background && background !== "transparent" ? background : undefined;

export type Entrance = "rise" | "fade" | "blur" | "scale" | "none";

export function entranceStyle(
  entrance: Entrance,
  frame: number,
  fps: number,
  delay = 0,
): React.CSSProperties {
  if (entrance === "none") return {};
  const f = frame - delay;
  const s = spring({ frame: f, fps, config: { damping: 200, mass: 0.8 } });
  const opacity = interpolate(f, [0, 14], [0, 1], CLAMPED);
  switch (entrance) {
    case "rise":
      return { opacity, transform: `translateY(${(1 - s) * 40}px)` };
    case "blur":
      return { opacity, filter: `blur(${(1 - s) * 18}px)` };
    case "scale":
      return { opacity, transform: `scale(${0.86 + s * 0.14})` };
    default:
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
  return Math.min(
    interpolate(t, [fromT, fromT + fade.in], [0, 1], { ...CLAMPED, easing: fade.easeIn }),
    interpolate(t, [toT - fade.out, toT], [1, 0], { ...CLAMPED, easing: fade.easeOut }),
  );
}

export function exitOpacity(frame: number, duration: number, frames = 10): number {
  return interpolate(frame, [duration - frames, duration], [1, 0], {
    ...CLAMPED,
    easing: Easing.in(Easing.quad),
  });
}
