import { describe, expect, it } from "vitest";
import { thumbnailTime } from "./media";

describe("thumbnailTime", () => {
  it("takes the frame at the chosen time when the file is long enough", () => {
    expect(thumbnailTime(1000, { durationMs: 4000 })).toBe(1000);
  });

  it("takes a frame just before the end of a shorter file", () => {
    expect(thumbnailTime(1000, { durationMs: 500 })).toBe(400);
    expect(thumbnailTime(60_000, { durationMs: 4000 })).toBe(3900);
  });

  it("takes the only frame of a still image", () => {
    expect(thumbnailTime(1000, { durationMs: 0 })).toBe(0);
  });
});
