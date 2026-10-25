import { describe, expect, it } from "vitest";
import {
  formatGradient,
  insertStop,
  parseGradient,
  sortStops,
  splitTopLevel,
} from "#core/gradient";

describe("parseGradient", () => {
  it("reads an angle and stops with positions", () => {
    expect(parseGradient("linear-gradient(115deg, #e8b86d 0%, rgba(0, 0, 0, 0.5) 60%)")).toEqual({
      type: "linear",
      angle: 115,
      stops: [
        { color: "#e8b86d", at: 0 },
        { color: "rgba(0, 0, 0, 0.5)", at: 60 },
      ],
    });
  });

  it("spreads stops without positions and reads side keywords", () => {
    expect(parseGradient("linear-gradient(to right, red, oklch(0.6 0.1 200), blue)")).toEqual({
      type: "linear",
      angle: 90,
      stops: [
        { color: "red", at: 0 },
        { color: "oklch(0.6 0.1 200)", at: 50 },
        { color: "blue", at: 100 },
      ],
    });
  });

  it("reads a radial gradient", () => {
    expect(parseGradient("radial-gradient(circle, #fff, #000)")?.type).toBe("radial");
  });

  it("refuses anything else", () => {
    expect(parseGradient("#ffffff")).toBeNull();
    expect(parseGradient("linear-gradient(90deg, nope, #000)")).toBeNull();
  });
});

describe("formatGradient", () => {
  it("round-trips sorted stops", () => {
    const text = formatGradient(
      sortStops({
        type: "linear",
        angle: 135,
        stops: [
          { color: "#0f172a", at: 100 },
          { color: "#1e1b4b", at: 0 },
        ],
      }),
    );
    expect(text).toBe("linear-gradient(135deg, #1e1b4b 0%, #0f172a 100%)");
    expect(parseGradient(text)).not.toBeNull();
  });
});

describe("insertStop", () => {
  it("adds a stop in the colour of the next one, up to six", () => {
    const g = parseGradient("linear-gradient(90deg, #000 0%, #fff 100%)");
    if (!g) throw new Error("no gradient");
    expect(insertStop(g, 40).stops.at(-1)).toEqual({ color: "#fff", at: 40 });
    let full = g;
    for (let i = 0; i < 8; i++) full = insertStop(full, 50);
    expect(full.stops).toHaveLength(6);
  });
});

describe("splitTopLevel", () => {
  it("keeps commas inside parentheses", () => {
    expect(splitTopLevel("a, rgb(1, 2, 3) 5%, b")).toEqual(["a", "rgb(1, 2, 3) 5%", "b"]);
  });
});
