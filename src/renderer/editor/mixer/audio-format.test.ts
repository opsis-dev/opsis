import { describe, expect, it } from "vitest";
import { db, levelDb, signedDb } from "./audio-format";

describe("db", () => {
  it("writes silence as minus infinity and signs the rest with a typographic minus", () => {
    expect(db(-120)).toBe("−∞");
    expect(db(Number.NaN)).toBe("−∞");
    expect(db(1.24)).toBe("+1.2");
    expect(db(-3)).toBe("−3.0");
    expect(db(-0.01)).toBe("0.0");
  });

  it("keeps large reductions as numbers", () => {
    expect(signedDb(-70)).toBe("−70.0");
  });
});

describe("levelDb", () => {
  it("shows quiet levels down to the silence floor", () => {
    expect(levelDb(-72.3)).toBe("−72.3");
    expect(levelDb(-120)).toBe("−∞");
  });
});
