import { describe, expect, it } from "vitest";
import { scriptToCaptions } from "./text";

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
});
