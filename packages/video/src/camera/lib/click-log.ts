import type { Point, Rect, Viewport } from "@opsis/core/schema/actions";

export type ClickEvent = {
  label?: string;
  tMs: number;
  tDepartMs?: number;
  tDownMs?: number;
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
