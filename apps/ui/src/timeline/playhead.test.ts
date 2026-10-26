import { describe, expect, it } from "vitest";
import { activeAt, loopOf, wrapInLoop } from "./playhead";

describe("activeAt", () => {
  const spans = { b: { from: 30, to: 90 }, a: { from: 0, to: 60 }, c: { from: 90, to: 120 } };

  it("lists the nodes whose span holds the frame, sorted", () => {
    expect(activeAt(spans, 45)).toEqual(["a", "b"]);
    expect(activeAt(spans, 0)).toEqual(["a"]);
  });

  it("ends a span before its last frame", () => {
    expect(activeAt(spans, 60)).toEqual(["b"]);
    expect(activeAt(spans, 90)).toEqual(["c"]);
    expect(activeAt(spans, 120)).toEqual([]);
  });
});

describe("loopOf", () => {
  it("loops the whole film without in and out points", () => {
    expect(loopOf(null, null, 300)).toEqual({ from: 0, to: 300 });
  });

  it("plays the out point's frame before looping back to the in point", () => {
    expect(loopOf(30, 120, 300)).toEqual({ from: 30, to: 121 });
    expect(loopOf(30, null, 300)).toEqual({ from: 30, to: 300 });
    expect(loopOf(null, 120, 300)).toEqual({ from: 0, to: 121 });
  });
});

describe("wrapInLoop", () => {
  const loop = { from: 30, to: 121 };

  it("keeps a position inside the loop", () => {
    expect(wrapInLoop(30, loop)).toBe(30);
    expect(wrapInLoop(120.5, loop)).toBe(120.5);
  });

  it("carries past the end back to the in point", () => {
    expect(wrapInLoop(121, loop)).toBe(30);
    expect(wrapInLoop(123.5, loop)).toBe(32.5);
  });

  it("carries before the start, when playing backwards, to the end", () => {
    expect(wrapInLoop(29, loop)).toBe(120);
  });
});
