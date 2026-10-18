import { describe, expect, it } from "vitest";
import { ffmpegSeconds, hasSoundTrack } from "./codecs";

describe("codecs", () => {
  it("knows which formats carry sound", () => {
    expect(hasSoundTrack("gif")).toBe(false);
    expect(hasSoundTrack("h264")).toBe(true);
  });

  it("writes times as seconds with millisecond precision", () => {
    expect(ffmpegSeconds(1500)).toBe("1.500");
    expect(ffmpegSeconds(33.4)).toBe("0.033");
  });
});
