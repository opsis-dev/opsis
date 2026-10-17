import { describe, expect, it } from "vitest";
import { overlayScale, shapeInsets, zoomBoost } from "./frame";

describe("other shapes", () => {
  it("keeps the landscape frame free of extra margins", () => {
    expect(shapeInsets(1920, 1080)).toEqual({ top: 0, right: 0, bottom: 0, left: 0 });
  });

  it("keeps overlays clear of the top and bottom of a vertical frame", () => {
    const tall = shapeInsets(1080, 1920);
    expect(tall.bottom).toBeGreaterThan(tall.top);
    expect(tall.bottom).toBeCloseTo(0.16 * 1920);
    expect(shapeInsets(1920, 1920).bottom).toBeLessThan(tall.bottom);
  });

  it("sizes overlays by the long side, so a vertical frame isn't shrunk", () => {
    expect(overlayScale(1080, 1920)).toBe(overlayScale(1920, 1080));
  });

  it("zooms in harder the narrower the frame, within limits", () => {
    expect(zoomBoost(1.6, 1.6)).toBe(1);
    expect(zoomBoost(1.6, 16 / 9)).toBe(1);
    expect(zoomBoost(1.6, 1)).toBeCloseTo(Math.sqrt(1.6));
    expect(zoomBoost(16 / 9, 9 / 16)).toBeCloseTo(16 / 9);
    expect(zoomBoost(4, 0.25)).toBe(2);
  });
});
