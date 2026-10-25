import type { RecordingProps } from "@opsis/core/schema/recording";

export const WINDOW_FIT = 0.86;
export const WINDOW_RADIUS = 14;
const STAGE_FIT = 0.84;
const STAGE_GROUND = "#faf6f1";
const INK = "15, 23, 42";

export const ink = (alpha: number): string => `rgba(${INK}, ${alpha})`;

export const faded = (color: string, share: string): string =>
  `color-mix(in srgb, ${color} ${share}, transparent)`;

export type WindowShot = {
  framing: "window";
  windowFit: number;
  ground: string | null;
  zoom: boolean;
};

type ShotStyle = WindowShot | { framing: "fullbleed" };

export const SHOT_STYLES: Record<RecordingProps["style"], ShotStyle> = {
  classic: { framing: "window", windowFit: WINDOW_FIT, ground: null, zoom: true },
  proof: { framing: "fullbleed" },
  ledger: { framing: "fullbleed" },
  stage: { framing: "window", windowFit: STAGE_FIT, ground: STAGE_GROUND, zoom: false },
};
