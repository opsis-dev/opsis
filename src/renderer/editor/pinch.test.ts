import { describe, expect, it } from "vitest";
import { MAX_ZOOM, MIN_ZOOM, pinchViewport } from "./pinch";

const at = (zoom: number) => ({ x: 0, y: 0, zoom });
const zoomAfter = (zoom: number, deltaY: number) => pinchViewport(at(zoom), deltaY, 0, 0).zoom;

describe("pinch zoom", () => {
  it("zooms in when the fingers spread and out when they pinch", () => {
    expect(zoomAfter(1, -10)).toBeGreaterThan(1);
    expect(zoomAfter(1, 10)).toBeLessThan(1);
  });

  it("stays within the canvas zoom range", () => {
    expect(zoomAfter(1.9, -1000)).toBe(MAX_ZOOM);
    expect(zoomAfter(0.11, 1000)).toBe(MIN_ZOOM);
  });

  it("keeps the point under the fingers still", () => {
    const view = { x: 40, y: -20, zoom: 0.5 };
    const next = pinchViewport(view, -10, 300, 200);
    expect((300 - next.x) / next.zoom).toBeCloseTo((300 - view.x) / view.zoom);
    expect((200 - next.y) / next.zoom).toBeCloseTo((200 - view.y) / view.zoom);
  });
});
