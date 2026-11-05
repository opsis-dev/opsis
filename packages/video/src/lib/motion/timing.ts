import { Easing } from "@opsis/runtime/easing.ts";
const DEFAULT_FPS = 30;

export const EASING_ENTER = Easing.bezier(0.16, 1, 0.3, 1);

export const EASING_EXIT = Easing.in(Easing.cubic);

export function secondsToFrames(seconds: number, fps = DEFAULT_FPS): number {
  return Math.round(seconds * fps);
}

