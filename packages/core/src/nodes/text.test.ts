import { describe, expect, it } from "vitest";
import { emptyClip } from "#core/compile/clip";
import { parseParams } from "#core/nodes/params";
import { getNodeDef } from "#core/nodes/registry";
import { endsSentence, scriptToCaptions } from "#core/nodes/text";
import type { CaptionsValue } from "#core/schema/values";
import { compileClip, must, nodeCard } from "#core/test-fixtures";

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

describe("Show captions", () => {
  const captions: CaptionsValue = {
    type: "captions",
    captions: [{ text: "Hi", startMs: 0, endMs: 2500 }],
  };

  it("lasts until the last caption has been read", () => {
    expect(compileClip("text.captions", {}, { captions }).durationInFrames).toBe(90);
  });

  it("keeps the length of the clip it is shown over", () => {
    expect(
      compileClip("text.captions", {}, { captions, clip: emptyClip(30) }).durationInFrames,
    ).toBe(30);
  });

  it("warns when the captions run past the clip", () => {
    const texts = nodeCard(
      "text.captions",
      {},
      {
        inputCaptionsMs: { captions: 2500 },
        inputFrames: { clip: 30 },
      },
    ).map((r) => r.text);
    expect(texts).toContain("Runs 1.5 s past the clip, which cuts it off");
  });

  it("needs a script to make captions from", () => {
    expect(
      parseParams(must(getNodeDef("text.captionsFromText"), "node").params, { script: " " }),
    ).toEqual({
      ok: false,
      problem: "Write a script",
    });
  });
});
