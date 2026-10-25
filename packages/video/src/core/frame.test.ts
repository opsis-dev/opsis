import { describe, expect, it } from "vitest";
import { sameShape, shapeInsets, zoomBoost } from "#video/core/frame.ts";

describe("other shapes", () => {
  it("adds margins only outside landscape, and more the narrower the frame", () => {
    const landscape = shapeInsets(1920, 1080);
    const square = shapeInsets(1920, 1920);
    const tall = shapeInsets(1080, 1920);
    expect(landscape.bottom).toBeLessThan(square.bottom);
    expect(square.bottom).toBeLessThan(tall.bottom);
  });

  it("keeps overlays clear of the bottom of a vertical frame more than the top", () => {
    const tall = shapeInsets(1080, 1920);
    expect(tall.bottom).toBeGreaterThan(tall.top);
  });

  it("treats nearly equal shapes as the same", () => {
    expect(sameShape({ width: 1920, height: 1080 }, { width: 1280, height: 720 })).toBe(true);
    expect(sameShape({ width: 1440, height: 900 }, { width: 1920, height: 1080 })).toBe(false);
  });

  it("zooms in harder the narrower the frame, within limits", () => {
    expect(zoomBoost(1.6, 1.6)).toBe(1);
    expect(zoomBoost(1.6, 16 / 9)).toBe(1);
    expect(zoomBoost(1.6, 1)).toBeCloseTo(Math.sqrt(1.6));
    expect(zoomBoost(16 / 9, 9 / 16)).toBeCloseTo(16 / 9);
    expect(zoomBoost(4, 0.25)).toBe(2);
  });
});
