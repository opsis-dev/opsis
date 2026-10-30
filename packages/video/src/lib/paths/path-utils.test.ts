import * as fc from "fast-check";
import { expect, it } from "vitest";
import {
  getPathDrawStyles,
  pathBoundingBox,
  pathLength,
  samplePath,
  waypointsToPath,
} from "#video/lib/paths/path-utils.ts";

const coordinate = fc.double({ min: -500, max: 500, noNaN: true });
const point = fc.record({ x: coordinate, y: coordinate });
const polyline = fc
  .array(point, { minLength: 2, maxLength: 8 })
  .filter((points) =>
    points.slice(1).every((p, i) => Math.hypot(p.x - points[i].x, p.y - points[i].y) > 1),
  );

function lineLengths(points: { x: number; y: number }[]): number[] {
  return points.slice(1).map((p, i) => Math.hypot(p.x - points[i].x, p.y - points[i].y));
}

it("measures a polyline as the sum of its sides", () => {
  fc.assert(
    fc.property(polyline, (points) => {
      const total = lineLengths(points).reduce((sum, side) => sum + side, 0);
      expect(pathLength(waypointsToPath(points, 0))).toBeCloseTo(total, 6);
    }),
  );
});

it("rides the first side of a polyline facing along it", () => {
  fc.assert(
    fc.property(polyline, fc.double({ min: 0.2, max: 0.8, noNaN: true }), (points, along) => {
      const d = waypointsToPath(points, 0);
      const sides = lineLengths(points);
      const total = sides.reduce((sum, side) => sum + side, 0);
      const sample = samplePath(d, (sides[0] * along) / total);
      const expected = Math.atan2(points[1].y - points[0].y, points[1].x - points[0].x);
      const angle = (sample.angle * Math.PI) / 180;
      expect(Math.cos(angle - expected)).toBeCloseTo(1, 6);
      expect(sample.x).toBeCloseTo(points[0].x + (points[1].x - points[0].x) * along, 6);
      expect(sample.y).toBeCloseTo(points[0].y + (points[1].y - points[0].y) * along, 6);
    }),
  );
});

it("faces the side it is about to ride at a corner, and the last side at the end", () => {
  fc.assert(
    fc.property(polyline, (points) => {
      fc.pre(points.length >= 3);
      const d = waypointsToPath(points, 0);
      const sides = lineLengths(points);
      const total = sides.reduce((sum, side) => sum + side, 0);
      function heading(from: number) {
        return Math.atan2(points[from + 1].y - points[from].y, points[from + 1].x - points[from].x);
      }
      const corner = (samplePath(d, sides[0] / total).angle * Math.PI) / 180;
      const end = (samplePath(d, 1).angle * Math.PI) / 180;
      expect(Math.cos(corner - heading(1))).toBeCloseTo(1, 4);
      expect(Math.cos(end - heading(points.length - 2))).toBeCloseTo(1, 4);
    }),
  );
});

it("bounds every point along a smooth path", () => {
  fc.assert(
    fc.property(polyline, fc.double({ min: 0.05, max: 1, noNaN: true }), (points, smoothing) => {
      const d = waypointsToPath(points, smoothing);
      const box = pathBoundingBox(d);
      const samples = Array.from({ length: 65 }, (_, i) => samplePath(d, i / 64));
      const slack = 1e-6 * (1 + Math.abs(box.x2 - box.x1) + Math.abs(box.y2 - box.y1));
      for (const sample of samples) {
        expect(sample.x).toBeGreaterThanOrEqual(box.x1 - slack);
        expect(sample.x).toBeLessThanOrEqual(box.x2 + slack);
        expect(sample.y).toBeGreaterThanOrEqual(box.y1 - slack);
        expect(sample.y).toBeLessThanOrEqual(box.y2 + slack);
      }
    }),
  );
});

it("draws a path on from hidden to whole", () => {
  fc.assert(
    fc.property(polyline, fc.double({ min: 0, max: 1, noNaN: true }), (points, progress) => {
      const d = waypointsToPath(points, 0);
      const length = pathLength(d);
      const styles = getPathDrawStyles(progress, d);
      const [dash] = styles.strokeDasharray.split(" ").map(Number);
      expect(dash).toBeGreaterThanOrEqual(length);
      expect(dash - styles.strokeDashoffset).toBeCloseTo(progress === 0 ? 0 : progress * length, 6);
    }),
  );
});
