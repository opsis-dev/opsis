import type { TimelineItem, TimelineSpec } from "#video/generated/spec.ts";

export const EMPTY_SPEC: TimelineSpec = {
  width: 1920,
  height: 1080,
  fps: 30,
  durationInFrames: 90,
  tracks: [],
  audio: [],
};

export type ElementProps = Record<string, unknown>;

export const AUTHORED_FPS = 30;

export type ElementItem = Extract<TimelineItem, { kind: "element" }>;
