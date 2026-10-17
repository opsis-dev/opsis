import { evenPx } from "../../shared/math";

type Size = { width: number; height: number };

export type PageScale = Size & { scale: number };

export function scaledViewport(logical: Size, scale: number): Size {
  return { width: evenPx(logical.width * scale), height: evenPx(logical.height * scale) };
}

export function fitPageToScale({ scale, width, height }: PageScale): void {
  const viewportUnit = /([-+]?[0-9]*\.?[0-9]+)(dvh|svh|lvh|vh|dvw|svw|lvw|vw)\b/g;
  const startupFrames = 30;
  const reapplyMs = 250;

  const pin = (target: object, key: string, value: number) => {
    try {
      Object.defineProperty(target, key, { get: () => value, configurable: true });
    } catch {
      return;
    }
  };
  pin(window, "innerWidth", width);
  pin(window, "innerHeight", height);
  if (window.visualViewport) {
    pin(window.visualViewport, "width", width);
    pin(window.visualViewport, "height", height);
  }

  const toPx = (text: string) =>
    text.replace(
      viewportUnit,
      (_, amount: string, unit: string) =>
        `${(parseFloat(amount) / 100) * (unit.endsWith("h") ? height : width)}px`,
    );

  const rewriteStyle = (style: CSSStyleDeclaration) => {
    for (let i = 0; i < style.length; i++) {
      const prop = style[i];
      const value = style.getPropertyValue(prop);
      if (!value.includes("v")) continue;
      const next = toPx(value);
      if (next !== value) style.setProperty(prop, next, style.getPropertyPriority(prop));
    }
  };

  let seenSheets = 0;
  const rewriteSheets = () => {
    const sheets = document.styleSheets;
    if (sheets.length === seenSheets) return;
    seenSheets = sheets.length;
    for (const sheet of Array.from(sheets)) {
      let rules: CSSRuleList;
      try {
        rules = sheet.cssRules;
      } catch {
        continue;
      }
      for (const rule of Array.from(rules))
        if ("style" in rule && rule.style instanceof CSSStyleDeclaration) rewriteStyle(rule.style);
    }
  };

  const apply = () => {
    document.documentElement?.style.setProperty("zoom", String(scale));
    try {
      rewriteSheets();
    } catch {
      return;
    }
  };
  apply();
  document.addEventListener("DOMContentLoaded", apply);
  window.addEventListener("load", apply);
  let frames = 0;
  const tick = () => {
    apply();
    if (++frames < startupFrames) requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
  setInterval(apply, reapplyMs);
}
