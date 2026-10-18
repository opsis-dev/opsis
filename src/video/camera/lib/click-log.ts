import { pointRect, type Point, type Rect, type Viewport } from "../../../shared/schema/actions";
import { designScaleFor } from "./design-width";

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

export const defaultTargetRect = (point: Point, viewportWidth: number): Rect =>
  pointRect(point, designScaleFor(viewportWidth));
