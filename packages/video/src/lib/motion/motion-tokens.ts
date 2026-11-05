import { Easing } from "@opsis/runtime/easing.ts";
import { EASING_ENTER, EASING_EXIT, secondsToFrames } from "#video/lib/motion/timing.ts";

export const DURATION = {
  fast: secondsToFrames(0.4),
  normal: secondsToFrames(0.8),
  slow: secondsToFrames(1.2),
} as const;

export const DELAY = {
  none: 0,
  short: secondsToFrames(0.15),
  medium: secondsToFrames(0.3),
} as const;

export const STAGGER = {
  tight: 4,
  normal: 8,
  relaxed: 12,
} as const;

export const EASING = {
  enter: EASING_ENTER,
  exit: EASING_EXIT,
  editorial: Easing.bezier(0.45, 0, 0.55, 1),
  pop: Easing.bezier(0.34, 1.56, 0.64, 1),
} as const;
