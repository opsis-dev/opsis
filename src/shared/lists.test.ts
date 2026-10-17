import { describe, expect, it } from "vitest";
import { appendCapped, keepLast } from "./lists";

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
