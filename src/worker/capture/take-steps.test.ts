import { describe, expect, it } from "vitest";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { clickKindOf, setupLength } from "./take-steps";

const node = (type: string, params: Record<string, unknown> = {}): WorkflowNode => ({
  id: "n",
  type,
  position: { x: 0, y: 0 },
  params,
});

describe("what happens before the camera rolls", () => {
  it("keeps waits and invisible steps right after the first page off camera", () => {
    const steps = [
      node("browser.setState", { script: "1" }),
      node("browser.navigate", { url: "https://example.com" }),
      node("browser.wait", { mode: "element", target: "Board" }),
      node("browser.hideElements"),
      node("browser.click", { target: "Open" }),
      node("browser.wait", { mode: "element", target: "Panel" }),
    ];
    expect(setupLength(steps, {})).toBe(4);
  });

  it("films a fixed pause after the first page", () => {
    const steps = [
      node("browser.navigate", { url: "https://example.com" }),
      node("browser.wait", { mode: "ms", ms: 1500 }),
    ];
    expect(setupLength(steps, {})).toBe(1);
  });

  it("has no setup without a page to open", () => {
    expect(setupLength([node("browser.press", { keys: "a" })], {})).toBe(0);
  });
});

describe("clickKindOf", () => {
  it.each([
    ["browser.click", "click"],
    ["browser.type", "type"],
    ["browser.hover", "hover"],
    ["browser.drag", "drag"],
    ["browser.select", "focus"],
  ])("logs a %s as a %s", (type, kind) => {
    expect(clickKindOf(type)).toBe(kind);
  });
});
