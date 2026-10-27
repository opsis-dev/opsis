import type { ClickButton, Point, Rect, Viewport } from "#video/generated/spec.ts";

export type ClickEvent = {
  label?: string;
  tMs: number;
  tDepartMs?: number;
  tDownMs?: number;
  tSecondDownMs?: number;
  button?: ClickButton;
  typeEndMs?: number;
  x: number;
  y: number;
  rect?: Rect;
  cluster?: string;
  zoom?: boolean;
  zoomScale?: number;
  tUpMs?: number;
  path?: Point[];
  drop?: Rect;
};

export type CursorSample = {
  t: number;
  x: number;
  y: number;
  c?: string;
};

export type KeyEvent = {
  tMs: number;
  chord: string;
  label?: string;
};

export type ClickLog = {
  name: string;
  viewport: Viewport;
  cursorTrack?: CursorSample[];
  durationMs: number;
  clicks: ClickEvent[];
  keys?: KeyEvent[];
};
