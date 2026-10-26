import { describe, expect, it } from "vitest";
import { mixerBook } from "@/store/catalog";
import { stripSettings } from "./strip-settings";

describe("strip settings", () => {
  it("copies a strip's settings without its routing", () => {
    const plain = mixerBook().defaultStrip;
    const strip = { ...plain, trim: 3, pan: -0.5, mute: true, solo: true, bus: "music" };
    expect(stripSettings(strip)).toEqual({ ...stripSettings(plain), trim: 3, pan: -0.5 });
  });
});
