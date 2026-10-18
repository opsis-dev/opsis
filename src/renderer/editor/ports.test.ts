import { describe, expect, it } from "vitest";
import { portLabel } from "./ports";

describe("portLabel", () => {
  it("names session ports after the action order", () => {
    expect(portLabel({ type: "session" }, "in")).toBe("Previous");
    expect(portLabel({ type: "session" }, "out")).toBe("Next");
  });

  it("names other ports after their type", () => {
    expect(portLabel({ type: "audio" }, "in")).toBe("Sound");
  });

  it("prefers the port's own label", () => {
    expect(portLabel({ type: "clip", label: "Intro" }, "in")).toBe("Intro");
  });
});
