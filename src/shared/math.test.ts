import { describe, expect, it } from "vitest";
import { clamp, clamp01, round3 } from "./math";

describe("math", () => {
  it("clamps into a range", () => {
    expect(clamp(5, 0, 10)).toBe(5);
    expect(clamp(-1, 0, 10)).toBe(0);
    expect(clamp(11, 0, 10)).toBe(10);
    expect(clamp01(1.5)).toBe(1);
  });

  it("rounds to three decimals", () => {
    expect(round3(1.23456)).toBe(1.235);
    expect(round3(29.97002997)).toBe(29.97);
  });
});
