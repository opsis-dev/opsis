import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { chordGlyphs } from "./click-log";

describe("chordGlyphs", () => {
  it("renders the reference's own chord", () => {
    assert.equal(chordGlyphs("Alt+Enter"), "⌥⏎");
  });

  it("keeps modifier order as authored, so it reads like the app's hint", () => {
    assert.equal(chordGlyphs("Meta+Shift+K"), "⌘⇧K");
  });

  it("upper-cases a bare letter and passes through unknown names", () => {
    assert.equal(chordGlyphs("k"), "K");
    assert.equal(chordGlyphs("F5"), "F5");
    assert.equal(chordGlyphs("constructor+K"), "CONSTRUCTORK");
  });

  it("tolerates spacing around the plus", () => {
    assert.equal(chordGlyphs("Alt + Enter"), "⌥⏎");
  });

  it("keeps the plus key itself", () => {
    assert.equal(chordGlyphs("Shift++"), "⇧+");
  });
});
