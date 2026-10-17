import { describe, expect, it } from "vitest";
import { framesScale, MS_SCALE, parseShown, shownText, unitScale } from "./decimal";

describe("seconds conversions", () => {
  it("shows stored milliseconds and frames in seconds", () => {
    expect(shownText(1500, MS_SCALE)).toBe("1.5");
    expect(shownText(45, framesScale(30))).toBe("1.5");
  });

  it("rounds typed seconds to whole stored units", () => {
    expect(parseShown("1.2345", MS_SCALE)).toBe(1235);
    expect(parseShown("0,5", framesScale(30))).toBe(15);
    expect(parseShown("", MS_SCALE)).toBeNull();
  });

  it("keeps other units unscaled and unrounded", () => {
    expect(parseShown("1.25", unitScale("px", 30))).toBe(1.25);
  });
});
