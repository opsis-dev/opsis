import { describe, expect, it } from "vitest";
import {
  chapterTime,
  formatTime,
  framesToMs,
  framesToRecordingMs,
  msToFrames,
  playRange,
  recordingMsToFrames,
  wholeFrames,
} from "#core/timeline/time";

describe("ms and frames", () => {
  it.each([
    ["1000 ms", msToFrames(1000, 30), 30],
    ["0 ms, at least one frame", msToFrames(0, 30), 1],
    ["0 ms, no minimum", msToFrames(0, 30, 0), 0],
    ["2.6 frames", wholeFrames(2.6), 3],
    ["a negative length", wholeFrames(-4), 0],
    ["0.2 frames, at least one", wholeFrames(0.2, 1), 1],
  ])("rounds %s to whole frames", (_case, frames, expected) => {
    expect(frames).toBe(expected);
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

  it("rounds before splitting minutes and seconds", () => {
    expect(formatTime(1799, 30, "seconds")).toBe("1:00");
    expect(formatTime(1797, 30, "seconds")).toBe("0:59.9");
    expect(formatTime(3, 30, "seconds")).toBe("0:00.1");
  });

  it("writes chapter timestamps", () => {
    expect(chapterTime(3_725_000)).toBe("1:02:05");
    expect(chapterTime(20_000)).toBe("0:20");
  });
});

describe("play range", () => {
  it("plays everything without in and out points", () => {
    expect(playRange(null, null, 100)).toEqual({});
  });

  it("keeps the out point after the in point and inside the film", () => {
    expect(playRange(10, null, 100)).toEqual({ inFrame: 10, outFrame: 99 });
    expect(playRange(null, 40, 100)).toEqual({ inFrame: 0, outFrame: 40 });
    expect(playRange(120, 130, 100)).toEqual({ inFrame: 98, outFrame: 99 });
  });

  it("has no range in a film too short for one", () => {
    expect(playRange(0, 0, 1)).toEqual({});
  });
});

describe("recording time", () => {
  it("places a recording moment on the timeline at the playback speed, and back", () => {
    expect(recordingMsToFrames(2000, 2, 30)).toBe(30);
    expect(recordingMsToFrames(1000, 0.5, 30)).toBe(60);
    expect(framesToRecordingMs(30, 2, 30)).toBe(2000);
  });
});
