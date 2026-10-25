import { describe, expect, it } from "vitest";
import { kindFromPath } from "#core/mime";

describe("kindFromPath", () => {
  it("reads every sound format as sound", () => {
    for (const ext of ["mp3", "wav", "opus", "aiff", "aif", "weba", "oga", "caf", "wma", "mka"])
      expect(kindFromPath(`assets/take.${ext.toUpperCase()}`)).toBe("audio");
  });

  it("reads pictures as images and the rest as video", () => {
    expect(kindFromPath("a/b.png")).toBe("image");
    expect(kindFromPath("a/b.mkv")).toBe("video");
    expect(kindFromPath("a/b.webm")).toBe("video");
  });
});
