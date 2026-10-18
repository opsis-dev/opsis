import { describe, expect, it } from "vitest";
import { emphasisedWords, groupCaptions } from "./text";

const words = (text: string) =>
  text.split(" ").map((word, i) => ({ text: word, startMs: i * 100, endMs: i * 100 + 90 }));

describe("groupCaptions", () => {
  it("ends a line at six words", () => {
    const lines = groupCaptions(words("one two three four five six seven"));
    expect(lines.map((line) => line.length)).toEqual([6, 1]);
  });

  it("ends a line at the end of a sentence", () => {
    const lines = groupCaptions(words("Ready? Set up in seconds. Go!"));
    expect(lines.map((line) => line.map((w) => w.text).join(" "))).toEqual([
      "Ready?",
      "Set up in seconds.",
      "Go!",
    ]);
  });

  it("returns no lines without captions", () => {
    expect(groupCaptions([])).toEqual([]);
  });
});

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
