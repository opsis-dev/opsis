import { describe, expect, it } from "vitest";
import type { PortType } from "@/shared/schema/workflow";
import { portLabel } from "./ports";

describe("portLabel", () => {
  it.each([
    ["session", "in", "Previous"],
    ["session", "out", "Next"],
    ["recording", "out", "Recording"],
    ["clip", "out", "Clip"],
    ["scene", "in", "Clip"],
    ["timeline", "out", "Clip"],
    ["audio", "in", "Sound"],
    ["captions", "out", "Captions"],
    ["image", "out", "Image"],
    ["media", "out", "File"],
    ["file", "in", "File"],
  ] as const)("names a %s %s port %j", (type: PortType, side, label) => {
    expect(portLabel({ type }, side)).toBe(label);
  });

  it("prefers the port's own label", () => {
    expect(portLabel({ type: "clip", label: "Intro" }, "in")).toBe("Intro");
  });
});
