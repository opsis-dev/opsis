import { clamp01 } from "@/shared/math";

export function isEditingElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLInputElement)
    return !/^(checkbox|radio|button|range|submit|reset|color|file)$/.test(target.type);
  return target.isContentEditable || /^(TEXTAREA|SELECT)$/.test(target.tagName);
}

type Rect = Pick<DOMRect, "left" | "top" | "width" | "height">;

export const fractionIn = (rect: Rect, clientX: number, clientY: number) => ({
  x: clamp01((clientX - rect.left) / Math.max(1, rect.width)),
  y: clamp01((clientY - rect.top) / Math.max(1, rect.height)),
});
