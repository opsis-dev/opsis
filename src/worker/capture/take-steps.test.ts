import { describe, expect, it } from "vitest";
import { workflowNode } from "../../shared/test-fixtures";
import { clickKindOf, setupLength } from "./take-steps";

describe("what happens before the camera rolls", () => {
  it("keeps waits and invisible steps right after the first page off camera", () => {
    const steps = [
      workflowNode("browser.setState", { script: "1" }),
      workflowNode("browser.navigate", { url: "https://example.com" }),
      workflowNode("browser.wait", { mode: "element", target: "Board" }),
      workflowNode("browser.hideElements"),
      workflowNode("browser.click", { target: "Open" }),
      workflowNode("browser.wait", { mode: "element", target: "Panel" }),
    ];
    expect(setupLength(steps, {})).toBe(4);
  });

  it("films a fixed pause after the first page", () => {
    const steps = [
      workflowNode("browser.navigate", { url: "https://example.com" }),
      workflowNode("browser.wait", { mode: "ms", ms: 1500 }),
    ];
    expect(setupLength(steps, {})).toBe(1);
  });

  it("has no setup without a page to open", () => {
    expect(setupLength([workflowNode("browser.press", { keys: "a" })], {})).toBe(0);
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
