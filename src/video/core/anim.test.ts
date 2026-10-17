import { describe, expect, it } from "vitest";
import { fadeWindow } from "./anim";

describe("fadeWindow", () => {
  it("ramps in and out over the window", () => {
    const fade = { in: 2, out: 4 };
    expect(fadeWindow(0, 0, 10, fade)).toBe(0);
    expect(fadeWindow(1, 0, 10, fade)).toBe(0.5);
    expect(fadeWindow(5, 0, 10, fade)).toBe(1);
    expect(fadeWindow(8, 0, 10, fade)).toBe(0.5);
    expect(fadeWindow(10, 0, 10, fade)).toBe(0);
  });

  it("skips a ramp of zero length", () => {
    expect(fadeWindow(0, 0, 10, { in: 0, out: 5 })).toBe(1);
    expect(fadeWindow(10, 0, 10, { in: 5, out: 0 })).toBe(1);
  });
});
