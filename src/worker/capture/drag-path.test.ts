import { describe, expect, it } from "vitest";
import {
  boundsOf,
  dragDuration,
  glidePlan,
  nudgeToward,
  offsetPath,
  parsePath,
  polylineAt,
  thinPath,
} from "./drag-path";

describe("glidePlan", () => {
  it("moves along the same curve glides always used", () => {
    const glide = glidePlan({ x: 0, y: 0 }, { x: 300, y: 400 }, 1, { width: 1920, height: 1080 });
    expect(glide.durationMs).toBeCloseTo(500 / 0.55, 6);
    expect(glide.overshooting).toBe(false);
    expect(glide.at(0)).toEqual({ x: 0, y: 0 });
    expect(glide.at(0.5).x).toBeCloseTo(130, 6);
    expect(glide.at(0.5).y).toBeCloseTo(215, 6);
    expect(glide.at(1)).toEqual({ x: 300, y: 400 });
  });

  it("overshoots long glides and keeps a set travel time", () => {
    const glide = glidePlan(
      { x: 0, y: 0 },
      { x: 600, y: 800 },
      1,
      { width: 1920, height: 1080 },
      700,
    );
    expect(glide.durationMs).toBe(700);
    expect(glide.overshooting).toBe(true);
    expect(glide.aim.x).toBeCloseTo(615.6, 6);
    expect(glide.aim.y).toBeCloseTo(820.8, 6);
  });
});

describe("polylineAt", () => {
  it("walks a path at an even pace by distance", () => {
    const at = polylineAt([
      { x: 0, y: 0 },
      { x: 100, y: 0 },
      { x: 100, y: 300 },
    ]);
    expect(at(0)).toEqual({ x: 0, y: 0 });
    expect(at(0.25)).toEqual({ x: 100, y: 0 });
    expect(at(0.5)).toEqual({ x: 100, y: 100 });
    expect(at(1)).toEqual({ x: 100, y: 300 });
  });
});

describe("offsetPath", () => {
  it("turns offsets into page points from the start", () => {
    expect(offsetPath({ x: 50, y: 60 }, [{ x: 300, y: 200 }])).toEqual([
      { x: 50, y: 60 },
      { x: 350, y: 260 },
    ]);
    expect(
      offsetPath({ x: 50, y: 60 }, [
        { x: 0, y: 0 },
        { x: 10, y: -10 },
      ]),
    ).toEqual([
      { x: 50, y: 60 },
      { x: 60, y: 50 },
    ]);
  });
});

describe("parsePath", () => {
  it("reads one point per line and names a bad line", () => {
    expect(parsePath("0, 0\n\n40, 40\n-120.5,-80")).toEqual([
      { x: 0, y: 0 },
      { x: 40, y: 40 },
      { x: -120.5, y: -80 },
    ]);
    expect(() => parsePath("0, 0\n10, 10\nleft")).toThrow("Line 3 of the path isn't two numbers.");
    expect(() => parsePath("  \n")).toThrow("Add at least one point to the path.");
  });
});

describe("dragDuration", () => {
  it("keeps drag times between 0.6 and 2.5 seconds unless set", () => {
    const short = [
      { x: 0, y: 0 },
      { x: 10, y: 0 },
    ];
    const long = [
      { x: 0, y: 0 },
      { x: 5000, y: 0 },
    ];
    expect(dragDuration(short, 1)).toBe(600);
    expect(dragDuration(long, 1)).toBe(2500);
    expect(dragDuration(long, 1, 4000)).toBe(4000);
  });
});

describe("nudgeToward", () => {
  it("nudges a few pixels toward the drop", () => {
    const nudge = nudgeToward({ x: 0, y: 0 }, { x: 30, y: 40 }, 6);
    expect(nudge.x).toBeCloseTo(3.6, 9);
    expect(nudge.y).toBeCloseTo(4.8, 9);
  });
});

describe("thinPath", () => {
  it("thins long paths to 64 points, keeping both ends", () => {
    const points = Array.from({ length: 500 }, (_, i) => ({ x: i, y: i * 2 }));
    const thin = thinPath(points);
    expect(thin).toHaveLength(64);
    expect(thin[0]).toEqual({ x: 0, y: 0 });
    expect(thin.at(-1)).toEqual({ x: 499, y: 998 });
    expect(boundsOf(thin)).toEqual({ x: 0, y: 0, w: 499, h: 998 });
  });
});
