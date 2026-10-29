import { describe, expect, it } from "vitest";
import { lengthLimits, parseListNumbers } from "./list-values";

describe("lengthLimits", () => {
  it("reads the array's min and max", () => {
    expect(lengthLimits({ type: "array", minItems: 2, maxItems: 7 })).toEqual({ min: 2, max: 7 });
  });

  it("is open without checks", () => {
    expect(lengthLimits({ type: "array" })).toEqual({ min: 0, max: Infinity });
  });
});

describe("parseListNumbers", () => {
  it("reads a spreadsheet column, keeping thousands separators", () => {
    expect(parseListNumbers("12,400\n9,100\n$320")).toEqual([12400, 9100, 320]);
  });

  it("reads one line of numbers", () => {
    expect(parseListNumbers("1, 2.5; 3 4")).toEqual([1, 2.5, 3, 4]);
  });

  it("reads a pasted row", () => {
    expect(parseListNumbers("5\t6\t7")).toEqual([5, 6, 7]);
  });
});
