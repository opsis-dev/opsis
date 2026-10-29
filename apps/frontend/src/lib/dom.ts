import { clamp01 } from "@opsis/video/common/math";
import type { KeyboardEvent as ReactKeyboardEvent, PointerEvent as ReactPointerEvent } from "react";

export function isEditingElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLInputElement)
    return !/^(checkbox|radio|button|range|submit|reset|color|file)$/.test(target.type);
  return target.isContentEditable || /^(TEXTAREA|SELECT)$/.test(target.tagName);
}

export const DRAG_START_PX = 3;

type Rect = Pick<DOMRect, "left" | "top" | "width" | "height">;

export const fractionIn = (rect: Rect, clientX: number, clientY: number) => ({
  x: clamp01((clientX - rect.left) / Math.max(1, rect.width)),
  y: clamp01((clientY - rect.top) / Math.max(1, rect.height)),
});

export function capturePrimaryPointer(e: ReactPointerEvent<Element>): boolean {
  if (e.button !== 0) return false;
  e.currentTarget.setPointerCapture(e.pointerId);
  return true;
}

export type ThresholdDrag = { started: boolean };

export function startOnceMoved(d: ThresholdDrag, distance: number, begin: () => void): boolean {
  if (d.started) return true;
  if (distance < DRAG_START_PX) return false;
  d.started = true;
  begin();
  return true;
}

export function endIfStarted(d: ThresholdDrag, end: () => void): boolean {
  if (d.started) end();
  return d.started;
}

function arrowDirection(key: string): -1 | 0 | 1 {
  if (key === "ArrowUp" || key === "ArrowRight") return 1;
  if (key === "ArrowDown" || key === "ArrowLeft") return -1;
  return 0;
}

export function onArrowKey(e: ReactKeyboardEvent, step: (dir: -1 | 1, big: boolean) => void) {
  const dir = arrowDirection(e.key);
  if (!dir) return;
  e.preventDefault();
  e.stopPropagation();
  step(dir, e.shiftKey);
}
