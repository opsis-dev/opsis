import { describe, expect, it } from "vitest";
import { DEFAULT_SETTINGS, mergeSettings } from "./settings";

describe("mergeSettings", () => {
  it("patches one section and keeps every other section", () => {
    expect(mergeSettings(DEFAULT_SETTINGS, { live: { fps: 24 } })).toEqual({
      ...DEFAULT_SETTINGS,
      live: { ...DEFAULT_SETTINGS.live, fps: 24 },
    });
  });

  it("refuses a value outside its range", () => {
    expect(() => mergeSettings(DEFAULT_SETTINGS, { live: { fps: 0 } })).toThrow("Too small");
  });
});
