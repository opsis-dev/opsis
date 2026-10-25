import { describe, expect, it } from "vitest";
import { groupCaptions } from "#video/core/captions.tsx";

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
