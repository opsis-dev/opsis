import assert from "node:assert/strict";
import { describe, it } from "vitest";
import type { KeyEvent } from "./click-log";
import { KEYCAP_HOLD_S, keycapAt, settle } from "./keycap";

describe("settle", () => {
  it("runs 1 -> 0 and clamps outside the move", () => {
    assert.equal(settle(0), 1);
    assert.equal(settle(1), 0);
    assert.equal(settle(-3), 1);
    assert.equal(settle(9), 0);
  });

  it("never rises on the way down", () => {
    let prev = settle(0);
    for (let i = 1; i <= 60; i++) {
      const v = settle(i / 60);
      assert.ok(v <= prev + 1e-9, `rose at u=${i / 60}: ${prev} -> ${v}`);
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
    assert.equal(keycapAt(keys, 0.5), null);
  });

  it("shows a chord for its hold window and then stops", () => {
    assert.equal(keycapAt(keys, 1.2)?.chord, "Alt+Enter");
    assert.equal(keycapAt(keys, 1 + KEYCAP_HOLD_S - 0.01)?.chord, "Alt+Enter");
    assert.equal(keycapAt(keys, 1 + KEYCAP_HOLD_S + 0.01), null);
  });

  it("replaces rather than stacks when chords overlap", () => {
    const fast: KeyEvent[] = [
      { tMs: 1000, chord: "Alt+Enter" },
      { tMs: 1200, chord: "Meta+K" },
    ];
    assert.equal(keycapAt(fast, 1.3)?.chord, "Meta+K");
  });

  it("ramps in at the start and out at the end of the window", () => {
    const first = keycapAt(keys, 1.001);
    assert.ok(first && first.inP < 0.2, "just-pressed should be arriving");
    const last = keycapAt(keys, 1 + KEYCAP_HOLD_S - 0.01);
    assert.ok(last && last.outP < 0.2, "about to expire should be leaving");
    const mid = keycapAt(keys, 1.5);
    assert.ok(mid && mid.inP === 1 && mid.outP === 1, "mid-hold is fully up");
  });
});
