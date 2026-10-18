import type { Locator } from "playwright-core";
import type { Point } from "../../shared/schema/actions";
import type { ClickEvent, CursorSample, KeyEvent } from "../../video/camera/lib/click-log";

export type TakeClick = ClickEvent & { selector?: string; text?: string };

export type TakeSink = {
  click: (event: TakeClick) => void;
  cursor: (sample: CursorSample) => void;
  key: (event: KeyEvent) => void;
};

export const CSS_PREFIX = "css=";

export type Target<L = Locator> = Point | L;

export const isPoint = (target: Target): target is Point => "x" in target;

export function elementOf(target: Target): Locator {
  if (isPoint(target))
    throw new Error("A point on the page isn't an element. Pick an element instead.");
  return target;
}

export type ActionOpts = {
  selector?: string;
  cluster?: string;
  zoom?: boolean;
  frame?: Target;
  chunkPauseMs?: number;
  zoomScale?: number;
  travelMs?: number;
  hoverMs?: number;
};

export type DragTo = { target: Target } | { by: Point } | { path: Point[] };

export type DragOpts = ActionOpts & {
  holdMs?: number;
  dragMs?: number;
  method?: "smooth" | "quick";
};
