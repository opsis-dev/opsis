import type { Locator, Page } from "playwright-core";
import type { ClickEvent } from "../../video/camera/lib/click-log";

export type { ClickEvent };

export type CssTarget = { css: string };

export const css = (selector: string): CssTarget => ({ css: selector });

export type Target = string | CssTarget | { x: number; y: number } | Locator;

export type ActionOpts = {
  cluster?: string;
  zoom?: boolean;
  frame?: Target;
  chunkPauseMs?: number;
  zoomScale?: number;
  travelMs?: number;
  hoverMs?: number;
};

export type DragTo =
  | { target: Target }
  | { by: { x: number; y: number } }
  | { path: Array<{ x: number; y: number }> };

export type DragOpts = ActionOpts & {
  holdMs?: number;
  dragMs?: number;
  method?: "smooth" | "quick";
};

export type FlowContext = {
  page: Page;
  baseURL: string;
  moveAndClick: (target: Target, label?: string, opts?: ActionOpts) => Promise<void>;
  typeInto: (
    target: Target,
    text: string | readonly string[],
    label?: string,
    opts?: ActionOpts,
  ) => Promise<void>;
  moveTo: (target: Target) => Promise<void>;
  drag: (from: Target, to: DragTo, label?: string, opts?: DragOpts) => Promise<void>;
  focus: (target: Target, label?: string, opts?: ActionOpts) => Promise<void>;
  find: (name: string) => Promise<Locator>;
  pause: (ms?: number) => Promise<void>;
  pressKey: (chord: string, label?: string) => Promise<void>;
  page_waitForURL: Page["waitForURL"];
};
