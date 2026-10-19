import { describe, expect, it } from "vitest";
import { formatDb, formatLevelDb, formatSignedDb } from "./audio-format";

describe("formatDb", () => {
  it("writes silence as minus infinity and signs the rest with a typographic minus", () => {
    expect(formatDb(-120)).toBe("−∞");
    expect(formatDb(Number.NaN)).toBe("−∞");
    expect(formatDb(1.24)).toBe("+1.2");
    expect(formatDb(-3)).toBe("−3.0");
    expect(formatDb(-0.01)).toBe("0.0");
  });
});

describe("formatSignedDb", () => {
  it("keeps large reductions as numbers", () => {
    expect(formatSignedDb(-70)).toBe("−70.0");
  });
});

describe("formatLevelDb", () => {
  it("shows quiet levels down to the silence floor", () => {
    expect(formatLevelDb(-72.3)).toBe("−72.3");
    expect(formatLevelDb(-120)).toBe("−∞");
  });
});
