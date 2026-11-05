import type { CameraStyle } from "#video/generated/spec.ts";

const WINDOW_FIT = 0.86;
export const WINDOW_RADIUS = 14;
const STAGE_FIT = 0.84;
const STAGE_GROUND = "#faf6f1";
const INK = "15, 23, 42";

export function ink(alpha: number): string {
  return `rgba(${INK}, ${alpha})`;
}

export function faded(color: string, share: string): string {
  return `color-mix(in srgb, ${color} ${share}, transparent)`;
}

export type WindowShot = {
  framing: "window";
  windowFit: number;
  ground: string | null;
  zoom: boolean;
};

type ShotStyle = WindowShot | { framing: "fullbleed" };

export const SHOT_STYLES: Record<CameraStyle, ShotStyle> = {
  classic: { framing: "window", windowFit: WINDOW_FIT, ground: null, zoom: true },
  proof: { framing: "fullbleed" },
  ledger: { framing: "fullbleed" },
  stage: { framing: "window", windowFit: STAGE_FIT, ground: STAGE_GROUND, zoom: false },
};
