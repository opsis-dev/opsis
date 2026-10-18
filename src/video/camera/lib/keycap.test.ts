import { describe, expect, it } from "vitest";
import type { KeyEvent } from "./click-log";
import { chordGlyphs, KEYCAP_HOLD_S, keycapAt, keycapEase } from "./keycap";

describe("chordGlyphs", () => {
  it.each([
    ["Alt+Enter", "⌥⏎"],
    ["Meta+Shift+K", "⌘⇧K"],
    ["Alt + Enter", "⌥⏎"],
    ["k", "K"],
    ["F5", "F5"],
    ["constructor+K", "CONSTRUCTORK"],
    ["Shift++", "⇧+"],
  ])("shows %s as %s, in the order written", (chord, glyphs) => {
    expect(chordGlyphs(chord)).toBe(glyphs);
  });
});

describe("keycapEase", () => {
  it("runs 0 -> 1 and clamps outside the move", () => {
    expect(keycapEase(0)).toBe(0);
    expect(keycapEase(1)).toBe(1);
    expect(keycapEase(-3)).toBe(0);
    expect(keycapEase(9)).toBe(1);
  });

  it("never falls on the way up", () => {
    let prev = keycapEase(0);
    for (let i = 1; i <= 60; i++) {
      const v = keycapEase(i / 60);
      expect(v).toBeGreaterThanOrEqual(prev - 1e-9);
      prev = v;
    }
  });
});

describe("keycapAt", () => {
  const keys: KeyEvent[] = [
    { tMs: 1000, chord: "Alt+Enter" },
    { tMs: 4000, chord: "Meta+K" },
  ];

  it("shows nothing before the first press", () => {
    expect(keycapAt(keys, 0.5)).toBe(null);
  });

  it("shows a chord for its hold window and then stops", () => {
    expect(keycapAt(keys, 1.2)?.chord).toBe("Alt+Enter");
    expect(keycapAt(keys, 1 + KEYCAP_HOLD_S - 0.01)?.chord).toBe("Alt+Enter");
    expect(keycapAt(keys, 1 + KEYCAP_HOLD_S + 0.01)).toBe(null);
  });

  it("replaces rather than stacks when chords overlap", () => {
    const fast: KeyEvent[] = [
      { tMs: 1000, chord: "Alt+Enter" },
      { tMs: 1200, chord: "Meta+K" },
    ];
    expect(keycapAt(fast, 1.3)?.chord).toBe("Meta+K");
  });

  it("ramps in at the start and out at the end of the window", () => {
    const first = keycapAt(keys, 1.001);
    expect(first && first.inP < 0.2, "just-pressed should be arriving").toBe(true);
    const last = keycapAt(keys, 1 + KEYCAP_HOLD_S - 0.01);
    expect(last && last.outP < 0.2, "about to expire should be leaving").toBe(true);
    const mid = keycapAt(keys, 1.5);
    expect(mid && mid.inP === 1 && mid.outP === 1, "mid-hold is fully up").toBe(true);
  });
});
