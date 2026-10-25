import { describe, expect, it } from "vitest";
import { isWallTime, wallTimeIn } from "#core/wall-time";

describe("wall time in a time zone", () => {
  it("reads the time as a clock in that zone shows it", () => {
    expect(wallTimeIn("2026-10-01T09:41", "Asia/Tokyo").toISOString()).toBe(
      "2026-10-01T00:41:00.000Z",
    );
    expect(wallTimeIn("2026-10-01T09:41:30", "America/New_York").toISOString()).toBe(
      "2026-10-01T13:41:30.000Z",
    );
    expect(wallTimeIn("2026-01-15T09:41", "America/New_York").toISOString()).toBe(
      "2026-01-15T14:41:00.000Z",
    );
    expect(wallTimeIn("2026-10-01T09:41", "UTC").toISOString()).toBe("2026-10-01T09:41:00.000Z");
  });

  it("accepts only a real date and time", () => {
    expect(isWallTime("2026-10-01T09:41")).toBe(true);
    expect(isWallTime("2026-10-01T09:41:05")).toBe(true);
    expect(isWallTime("not a date")).toBe(false);
    expect(isWallTime("")).toBe(false);
    expect(isWallTime("2026-02-30T09:41")).toBe(false);
    expect(isWallTime("2026-10-01T25:00")).toBe(false);
    expect(() => wallTimeIn("soon", "UTC")).toThrow("“soon” isn't a date and time.");
  });
});
