import { evenPx } from "@opsis/core/math";
import type { Size } from "@opsis/video/capture";

type PageScale = Size & { scale: number };

export function scaledViewport(logical: Size, scale: number): Size {
  return { width: evenPx(logical.width * scale), height: evenPx(logical.height * scale) };
}

export function fitPageToScale({ scale, width, height }: PageScale): void {
  const viewportUnit = /([-+]?[0-9]*\.?[0-9]+)(dvh|svh|lvh|vh|dvw|svw|lvw|vw)\b/g;
  const startupFrames = 30;
  const reapplyMs = 250;
  const zoom = String(scale);

  function pin(target: object, key: string, value: number) {
    try {
      Object.defineProperty(target, key, { get: () => value, configurable: true });
    } catch {}
  }

  function pinViewport() {
    pin(window, "innerWidth", width);
    pin(window, "innerHeight", height);
    if (!window.visualViewport) return;
    pin(window.visualViewport, "width", width);
    pin(window.visualViewport, "height", height);
  }

  const mayHaveViewportUnit = (value: string) => value.includes("v");

  function toPx(text: string) {
    return text.replace(
      viewportUnit,
      (_, amount: string, unit: string) =>
        `${(parseFloat(amount) / 100) * (unit.endsWith("h") ? height : width)}px`,
    );
  }

  function rewriteStyle(style: CSSStyleDeclaration) {
    for (let i = 0; i < style.length; i++) {
      const prop = style[i];
      const value = style.getPropertyValue(prop);
      if (!mayHaveViewportUnit(value)) continue;
      const next = toPx(value);
      if (next !== value) style.setProperty(prop, next, style.getPropertyPriority(prop));
    }
  }

  function readableRules(sheet: CSSStyleSheet): CSSRule[] {
    try {
      return Array.from(sheet.cssRules);
    } catch {
      return [];
    }
  }

  let seenSheets = 0;
  function rewriteNewSheets() {
    const sheets = document.styleSheets;
    if (sheets.length === seenSheets) return;
    seenSheets = sheets.length;
    for (const sheet of Array.from(sheets))
      for (const rule of readableRules(sheet))
        if ("style" in rule && rule.style instanceof CSSStyleDeclaration) rewriteStyle(rule.style);
  }

  function apply() {
    const root = document.documentElement;
    if (root && root.style.getPropertyValue("zoom") !== zoom) root.style.setProperty("zoom", zoom);
    rewriteNewSheets();
  }

  function applyForStartupFrames(frames = 0) {
    apply();
    if (frames + 1 < startupFrames) requestAnimationFrame(() => applyForStartupFrames(frames + 1));
  }

  pinViewport();
  document.addEventListener("DOMContentLoaded", apply);
  window.addEventListener("load", apply);
  setInterval(apply, reapplyMs);
  apply();
  requestAnimationFrame(() => applyForStartupFrames());
}
