import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import type { KeyEvent } from "#video/camera/lib/click-log.ts";
import { chordGlyphs, keycapAt, keycapEase, shownKeys } from "#video/camera/lib/keycap.ts";

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

  function lastShown(events: KeyEvent[], fromS: number) {
    let t = fromS;
    while (keycapAt(events, t + 0.005)) t += 0.005;
    return t;
  }

  it("shows a chord for about a second and then stops", () => {
    expect(keycapAt(keys, 1.2)?.chord).toBe("Alt+Enter");
    const end = lastShown(keys, 1);
    expect(end).toBeGreaterThan(1.8);
    expect(end).toBeLessThan(2.5);
    expect(keycapAt(keys, end + 0.01)).toBe(null);
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
    const last = keycapAt(keys, lastShown(keys, 1));
    expect(last && last.outP < 0.2, "about to expire should be leaving").toBe(true);
    const mid = keycapAt(keys, 1.5);
    expect(mid && mid.inP === 1 && mid.outP === 1, "mid-hold is fully up").toBe(true);
  });
});

const NAMED = ["Escape", "Enter", "Tab", "ArrowUp", "Backspace", "Delete", "Space"];
const SPELLINGS: Record<string, string[]> = {
  Escape: ["Escape", "esc", "ESC"],
  Enter: ["Enter", "Return", "enter"],
  Control: ["Control", "Ctrl", "ctrl"],
  Meta: ["Meta", "Cmd", "Command"],
};

function spelled(key: string, at: number): string {
  const ways = SPELLINGS[key] ?? [key, key.toLowerCase()];
  return ways[at % ways.length];
}

describe("shownKeys", () => {
  it("leaves out a listed key however it is spelled, and keeps it when another key is held", () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...NAMED),
        fc.constantFrom("Control", "Meta", "Shift", "Alt"),
        fc.nat(5),
        fc.nat(5),
        (key, held, a, b) => {
          const keys: KeyEvent[] = [
            { tMs: 0, chord: spelled(key, a) },
            { tMs: 100, chord: `${spelled(held, b)}+${spelled(key, a + b)}` },
            { tMs: 200, chord: "Control+K" },
          ];
          expect(shownKeys(keys, [spelled(key, b)]).map((k) => k.tMs)).toEqual([100, 200]);
          expect(
            shownKeys(keys, [`${spelled(key, a)}+${spelled(held, a)}`]).map((k) => k.tMs),
          ).toEqual([0, 200]);
        },
      ),
    );
  });

  it("shows every key when nothing is left out", () => {
    fc.assert(
      fc.property(fc.array(fc.constantFrom(...NAMED, "Control+K", "/")), (chords) => {
        const keys = chords.map((chord, tMs) => ({ tMs, chord }));
        expect(shownKeys(keys, [])).toEqual(keys);
      }),
    );
  });
});
