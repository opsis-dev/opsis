import { describe, expect, it } from "vitest";
import { formatColor, formatOf, parseColor, projectSwatches, sameColor } from "#core/color";

describe("parseColor", () => {
  it("keeps the alpha of rgba, hex8 and oklch values", () => {
    expect(parseColor("rgba(10,10,12,0.72)")?.a).toBeCloseTo(0.72);
    expect(parseColor("#ffffff80")?.a).toBeCloseTo(0.5, 1);
    expect(parseColor("oklch(1 0 0 / 10%)")?.a).toBeCloseTo(0.1);
    expect(parseColor("transparent")?.a).toBe(0);
  });

  it("refuses CSS variables and non-colours", () => {
    expect(parseColor("var(--accent)")).toBeNull();
    expect(parseColor("")).toBeNull();
    expect(parseColor("linear-gradient(red, blue)")).toBeNull();
  });
});

describe("formatColor", () => {
  const half = { h: 0, s: 1, v: 1, a: 0.5 };

  it("writes each format, with alpha only when it isn't opaque", () => {
    expect(formatColor({ ...half, a: 1 }, "hex")).toBe("#ff0000");
    expect(formatColor(half, "hex")).toBe("#ff000080");
    expect(formatColor(half, "rgb")).toBe("rgba(255, 0, 0, 0.5)");
    expect(formatColor({ ...half, a: 1 }, "hsl")).toBe("hsl(0, 100%, 50%)");
    expect(formatColor(half, "oklch")).toMatch(/^oklch\([\d.]+ [\d.]+ [\d.]+ \/ 0\.5\)$/);
  });

  it("round-trips through the same format", () => {
    for (const text of ["#26242c", "rgba(10, 10, 12, 0.72)", "hsl(200, 50%, 40%)"]) {
      const parsed = parseColor(text);
      expect(parsed && sameColor(formatColor(parsed, formatOf(text)), text)).toBe(true);
    }
  });
});

describe("formatOf", () => {
  it("names the format a value is written in", () => {
    expect(formatOf("#fff")).toBe("hex");
    expect(formatOf("rgba(0,0,0,0.3)")).toBe("rgb");
    expect(formatOf("hsla(1, 2%, 3%, 0.4)")).toBe("hsl");
    expect(formatOf("oklch(0.5 0.1 20)")).toBe("oklch");
  });
});

describe("projectSwatches", () => {
  it("lists colours used in params, most used first, once each", () => {
    const params = [
      { color: "#ff0000", text: "Hello #1" },
      { colors: ["#FF0000", "rgb(0, 0, 255)"], nested: { ink: "#ff0000" } },
    ];
    expect(projectSwatches(params)).toEqual(["#ff0000", "rgb(0, 0, 255)"]);
  });
});
