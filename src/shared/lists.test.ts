import { describe, expect, it } from "vitest";
import { appendCapped, firstFree, keepLast } from "./lists";

describe("lists", () => {
  it("keeps the last items", () => {
    expect(keepLast([1, 2, 3], 2)).toEqual([2, 3]);
    expect(keepLast([1, 2, 3], 5)).toEqual([1, 2, 3]);
    expect(keepLast([1, 2, 3], 0)).toEqual([]);
  });

  it("appends and drops the oldest past the limit", () => {
    expect(appendCapped([1, 2, 3], 4, 3)).toEqual([2, 3, 4]);
    expect(appendCapped([1], 2, 3)).toEqual([1, 2]);
    expect(appendCapped([1, 2], 3, 1)).toEqual([3]);
  });
});

describe("firstFree", () => {
  it("returns the first candidate that isn't taken", () => {
    const taken = new Set(["Variable 1", "Variable 3"]);
    expect(
      firstFree(
        (n) => `Variable ${n}`,
        (name) => taken.has(name),
      ),
    ).toBe("Variable 2");
    expect(
      firstFree(
        (n) => (n === 0 ? "a.mp4" : `a-${n}.mp4`),
        () => false,
        0,
      ),
    ).toBe("a.mp4");
  });
});
