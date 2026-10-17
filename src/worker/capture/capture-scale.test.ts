import { afterEach, describe, expect, it, vi } from "vitest";
import { fitPageToScale, scaledViewport } from "./capture-scale";

class FakeStyle {
  [index: number]: string;
  length = 0;
  private values = new Map<string, string>();

  constructor(entries: Record<string, string> = {}) {
    for (const [prop, value] of Object.entries(entries)) this.setProperty(prop, value);
  }

  getPropertyValue(prop: string) {
    return this.values.get(prop) ?? "";
  }

  getPropertyPriority() {
    return "";
  }

  setProperty(prop: string, value: string) {
    if (!this.values.has(prop)) this[this.length++] = prop;
    this.values.set(prop, value);
  }
}

function fakePage(rule: FakeStyle) {
  const listen = vi.fn();
  const win = { visualViewport: {}, addEventListener: listen } as Record<string, unknown>;
  const root = new FakeStyle();
  vi.stubGlobal("window", win);
  vi.stubGlobal("document", {
    documentElement: { style: root },
    styleSheets: [{ cssRules: [{ style: rule }, { cssText: "@import" }] }],
    addEventListener: listen,
  });
  vi.stubGlobal("CSSStyleDeclaration", FakeStyle);
  vi.stubGlobal("requestAnimationFrame", vi.fn());
  return { win, root };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("scaledViewport", () => {
  it("scales to even sizes", () => {
    expect(scaledViewport({ width: 1281, height: 721 }, 1.5)).toEqual({
      width: 1922,
      height: 1082,
    });
  });
});

describe("fitPageToScale", () => {
  it("keeps the page at its logical size while drawing it bigger", () => {
    vi.useFakeTimers();
    const rule = new FakeStyle({ height: "100vh", width: "50dvw", color: "red" });
    const { win, root } = fakePage(rule);
    fitPageToScale({ scale: 2, width: 1280, height: 720 });
    expect(win.innerWidth).toBe(1280);
    expect(win.innerHeight).toBe(720);
    expect((win.visualViewport as { width: number }).width).toBe(1280);
    expect(root.getPropertyValue("zoom")).toBe("2");
    expect(rule.getPropertyValue("height")).toBe("720px");
    expect(rule.getPropertyValue("width")).toBe("640px");
    expect(rule.getPropertyValue("color")).toBe("red");
  });
});
