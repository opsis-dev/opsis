import type { RecordingProps } from "../../../shared/compile/recording";

export const WINDOW_FIT = 0.86;

export type WindowShot = {
  framing: "window";
  windowFit: number;
  ground: string | null;
  zoom: boolean;
};

type ShotStyle = WindowShot | { framing: "fullbleed" | "isolate" };

export const SHOT_STYLES: Record<RecordingProps["style"], ShotStyle> = {
  classic: { framing: "window", windowFit: WINDOW_FIT, ground: null, zoom: true },
  proof: { framing: "fullbleed" },
  ledger: { framing: "isolate" },
  stage: { framing: "window", windowFit: 0.84, ground: "#faf6f1", zoom: false },
};
