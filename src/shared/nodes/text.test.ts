import { describe, expect, it } from "vitest";
import { endsSentence, scriptToCaptions } from "./text";

describe("scriptToCaptions", () => {
  it("times each word at the speaking pace and pauses after a sentence", () => {
    const captions = scriptToCaptions("Record it. Then  share", {
      wordsPerMinute: 120,
      startMs: 1000,
      linePauseMs: 300,
    });
    expect(captions).toEqual([
      { text: "Record", startMs: 1000, endMs: 1500 },
      { text: "it.", startMs: 1500, endMs: 2000 },
      { text: "Then", startMs: 2300, endMs: 2800 },
      { text: "share", startMs: 2800, endMs: 3300 },
    ]);
  });

  it("makes no captions from an empty script", () => {
    expect(scriptToCaptions("  ", { wordsPerMinute: 160, startMs: 0, linePauseMs: 0 })).toEqual([]);
  });

  const pace = { wordsPerMinute: 120, startMs: 0, linePauseMs: 300 };
  const starts = (script: string) => scriptToCaptions(script, pace).map((c) => [c.text, c.startMs]);

  it("ends sentences behind quotes, brackets and ellipses, but not after abbreviations", () => {
    expect(['it."', "(see it.)", "wait…", "Done!”", "end。"].every(endsSentence)).toBe(true);
    expect(["e.g.", "i.e.", "Dr.", "etc."].some(endsSentence)).toBe(false);
  });

  it("pauses longer at a paragraph break", () => {
    expect(starts("One\n\nTwo")).toEqual([
      ["One", 0],
      ["Two", 1100],
    ]);
  });

  it("splits text written without spaces into words", () => {
    expect(scriptToCaptions("今日はいい天気です。", pace).length).toBeGreaterThan(1);
  });
});
