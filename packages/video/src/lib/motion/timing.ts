import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { CLAMPED } from "#video/core/anim.ts";

export const DEFAULT_FPS = 30;

export const EASING_ENTER = Easing.bezier(0.16, 1, 0.3, 1);

export const EASING_EXIT = Easing.in(Easing.cubic);

export function secondsToFrames(seconds: number, fps = DEFAULT_FPS): number {
  return Math.round(seconds * fps);
}

export function framesToSeconds(frames: number, fps = DEFAULT_FPS): number {
  return frames / fps;
}

export function staggerDelay(
  index: number,
  staggerInFrames: number,
  baseDelayInFrames = 0,
): number {
  return baseDelayInFrames + index * staggerInFrames;
}

export function enterProgress(
  frame: number,
  delayInFrames: number,
  durationInFrames: number,
  easing = EASING_ENTER,
): number {
  return interpolate(frame, [delayInFrames, delayInFrames + durationInFrames], [0, 1], {
    ...CLAMPED,
    easing,
  });
}

export function exitProgress(
  frame: number,
  delayInFrames: number,
  durationInFrames: number,
  easing = EASING_EXIT,
): number {
  return interpolate(frame, [delayInFrames, delayInFrames + durationInFrames], [0, 1], {
    ...CLAMPED,
    easing,
  });
}
