import { describe, expect, it } from "vitest";
import { chapterTime, formatTime, framesToMs, msToFrames } from "./time";

describe("ms and frames", () => {
  it("rounds to whole frames with a minimum", () => {
    expect(msToFrames(1000, 30)).toBe(30);
    expect(msToFrames(0, 30)).toBe(1);
    expect(msToFrames(0, 30, 0)).toBe(0);
  });

  it("rounds frames to whole milliseconds", () => {
    expect(framesToMs(30, 30)).toBe(1000);
    expect(framesToMs(1, 30)).toBe(33);
  });
});

describe("time formats", () => {
  it("writes seconds and timecode", () => {
    expect(formatTime(0, 30, "seconds")).toBe("0:00");
    expect(formatTime(45, 30, "seconds")).toBe("0:01.5");
    expect(formatTime(30 * 75, 30, "seconds")).toBe("1:15");
    expect(formatTime(30 * 3661 + 12, 30, "timecode")).toBe("01:01:01:12");
  });

  it("writes chapter timestamps", () => {
    expect(chapterTime(3_725_000)).toBe("1:02:05");
    expect(chapterTime(20_000)).toBe("0:20");
  });
});
