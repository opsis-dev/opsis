import { describe, expect, it } from "vitest";
import { rulerStep, snapFrame, snapSpan } from "./snap";

describe("snapping", () => {
  const targets = [
    { frame: 30, kind: "playhead" as const },
    { frame: 90, kind: "edge" as const },
  ];

  it("snaps to the nearest target inside the threshold", () => {
    expect(snapFrame(33, targets, 5)).toEqual({ frame: 30, target: targets[0] });
    expect(snapFrame(40, targets, 5).target).toBeNull();
  });

  it("snaps whichever end of a moved span is closer", () => {
    expect(snapSpan(0, 50, 28, targets, 5).delta).toBe(30);
    expect(snapSpan(0, 50, 42, targets, 5).delta).toBe(40);
    expect(snapSpan(0, 50, 10, targets, 5)).toEqual({ delta: 10, target: null });
  });
});

describe("ruler", () => {
  it("spaces ruler ticks by zoom", () => {
    expect(rulerStep(30, 30)).toEqual({ step: 5, minor: 1 });
    expect(rulerStep(1, 30)).toEqual({ step: 150, minor: 30 });
    expect(rulerStep(0.05, 30)).toEqual({ step: 1800, minor: 360 });
    expect(rulerStep(0.001, 30)).toEqual({ step: 18_000, minor: 1800 });
  });
});
