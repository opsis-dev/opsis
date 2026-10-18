import { describe, expect, it } from "vitest";
import { emphasisedWords } from "./kinetic-headline";

describe("emphasisedWords", () => {
  it("marks words wrapped in asterisks and drops the asterisks", () => {
    expect(emphasisedWords("Ship *faster*  with *demos*")).toEqual([
      { word: "Ship", em: false },
      { word: "faster", em: true },
      { word: "with", em: false },
      { word: "demos", em: true },
    ]);
  });
});
