import { describe, expect, it } from "vitest";
import { formatTime, rulerStep, snapFrame, snapSpan } from "./snap";

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

describe("time formats", () => {
  it("writes seconds and timecode", () => {
    expect(formatTime(0, 30, "seconds")).toBe("0:00");
    expect(formatTime(45, 30, "seconds")).toBe("0:01.5");
    expect(formatTime(30 * 75, 30, "seconds")).toBe("1:15");
    expect(formatTime(30 * 3661 + 12, 30, "timecode")).toBe("01:01:01:12");
  });

  it("spaces ruler ticks by zoom", () => {
    expect(rulerStep(30, 30).step).toBe(5);
    expect(rulerStep(1, 30).step).toBe(150);
    expect(rulerStep(0.05, 30).step).toBe(1800);
  });
});
