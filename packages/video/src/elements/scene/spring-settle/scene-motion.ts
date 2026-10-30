import { Easing, interpolate, random } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";

export const EXPO = Easing.bezier(0.16, 1, 0.3, 1);

export const FADE = Easing.out(Easing.quad);

export const SETTLE = Easing.spring({ damping: 18, stiffness: 120, mass: 1 });

export const SETTLE_SOFT = Easing.spring({
  damping: 26,
  stiffness: 90,
  mass: 1,
});

export const SETTLE_MARK = Easing.spring({
  damping: 13,
  stiffness: 140,
  mass: 1,
});

export function stagger(
  index: number,
  base = 5,
  power = 0.8,
  jitter = 1.5,
  seed = "stagger",
): number {
  return Math.max(0, index ** power * base + (random(`${seed}-${index}`) - 0.5) * 2 * jitter);
}

export function fadeIn(frame: number, at: number, frames = 7): number {
  return interpolate(frame, [at, at + frames], [0, 1], { ...CLAMPED, easing: FADE });
}
