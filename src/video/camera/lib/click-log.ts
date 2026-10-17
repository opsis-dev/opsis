import { pointRect, type Rect } from "../../../shared/schema/actions";
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
  path?: Array<{ x: number; y: number }>;
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
  viewport: { width: number; height: number };
  cursorTrack?: CursorSample[];
  durationMs: number;
  clicks: ClickEvent[];
  keys?: KeyEvent[];
};

export const defaultTargetRect = (point: { x: number; y: number }, viewportWidth: number): Rect =>
  pointRect(point, designScaleFor(viewportWidth));
