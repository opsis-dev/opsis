import { describe, expect, it } from "vitest";
import { MAX_ZOOM, MIN_ZOOM, pinchViewport, pinchZoom } from "./pinch";

describe("pinch zoom", () => {
  it("zooms as fast as a pinch on macOS", () => {
    expect(pinchZoom(1, -10)).toBeCloseTo(2 ** 0.2);
    expect(pinchZoom(1, 10)).toBeCloseTo(2 ** -0.2);
  });

  it("limits one event and stays within the canvas zoom range", () => {
    expect(pinchZoom(1, -1000)).toBeCloseTo(2 ** 0.5);
    expect(pinchZoom(1.9, -100)).toBe(MAX_ZOOM);
    expect(pinchZoom(0.11, 100)).toBe(MIN_ZOOM);
  });

  it("keeps the point under the fingers still", () => {
    const view = { x: 40, y: -20, zoom: 0.5 };
    const next = pinchViewport(view, -10, 300, 200);
    expect((300 - next.x) / next.zoom).toBeCloseTo((300 - view.x) / view.zoom);
    expect((200 - next.y) / next.zoom).toBeCloseTo((200 - view.y) / view.zoom);
  });
});
