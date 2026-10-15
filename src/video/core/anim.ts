import type React from "react";
import { Easing, interpolate, spring } from "remotion";

export const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

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
  const opacity = interpolate(f, [0, 14], [0, 1], clamp);
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

export function exitOpacity(frame: number, duration: number, frames = 10): number {
  return interpolate(frame, [duration - frames, duration], [1, 0], {
    ...clamp,
    easing: Easing.in(Easing.quad),
  });
}
