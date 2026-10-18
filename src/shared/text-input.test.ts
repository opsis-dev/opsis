import { describe, expect, it } from "vitest";
import {
  chordKeys,
  formatNumberList,
  nonBlankLines,
  parseNumberList,
  parseNumberText,
} from "./text-input";

describe("parseNumberText", () => {
  it("reads comma decimals and gives nothing for blank text", () => {
    expect(parseNumberText("1,5")).toBe(1.5);
    expect(parseNumberText(" 2 ")).toBe(2);
    expect(parseNumberText("")).toBeNull();
    expect(parseNumberText("   ")).toBeNull();
    expect(parseNumberText("12 dB")).toBeNull();
  });

  it("ignores units when lenient", () => {
    expect(parseNumberText("-12 dB", { lenient: true })).toBe(-12);
    expect(parseNumberText("5s", { lenient: true })).toBe(5);
    expect(parseNumberText("dB", { lenient: true })).toBeNull();
  });
});

describe("number lists", () => {
  it("parses comma lists and drops trailing zeros when writing", () => {
    expect(parseNumberList("100, x,250")).toEqual([100, 0, 250]);
    expect(parseNumberList("")).toEqual([0]);
    expect(parseNumberList(undefined)).toEqual([0]);
    expect(formatNumberList([100, 0, 0])).toBe("100");
    expect(formatNumberList([0, 0])).toBe("");
  });
});

describe("nonBlankLines", () => {
  it("trims lines and drops empty ones", () => {
    expect(nonBlankLines(" a \n\n  \nb")).toEqual(["a", "b"]);
  });
});

describe("chordKeys", () => {
  it.each([
    ["Control+K", ["Control", "K"]],
    ["Shift++", ["Shift", "+"]],
    ["+", ["+"]],
    ["Alt + Enter", ["Alt", "Enter"]],
  ])("splits %j into its keys", (chord, keys) => {
    expect(chordKeys(chord)).toEqual(keys);
  });
});
