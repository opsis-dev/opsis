import { describe, expect, it } from "vitest";
import { DEFAULT_STRIP, stripSettings } from "./mix";

describe("strip settings", () => {
  it("copies a strip's settings without its routing", () => {
    const strip = { ...DEFAULT_STRIP, trim: 3, pan: -0.5, mute: true, solo: true, bus: "music" };
    expect(stripSettings(strip)).toEqual({ ...stripSettings(DEFAULT_STRIP), trim: 3, pan: -0.5 });
  });
});
