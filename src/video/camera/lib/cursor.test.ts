import assert from "node:assert/strict";
import { describe, it } from "vitest";
import {
  CLICK_SQUASH,
  CLICK_SQUASH_S,
  CURSOR_FADE_S,
  type ClickEvent,
  type CursorSample,
} from "./click-log";
import { cursorAt, cursorOpacity, samplePath } from "./cursor";

const track: CursorSample[] = [
  { t: 0, x: 100, y: 100 },
  { t: 500, x: 300, y: 200 },
  { t: 1000, x: 500, y: 400 },
];

describe("samplePath", () => {
  it("holds the ends outside the recorded range", () => {
    assert.deepEqual(samplePath(track, -5), { x: 100, y: 100 });
    assert.deepEqual(samplePath(track, 99), { x: 500, y: 400 });
  });

  it("interpolates between samples", () => {
    assert.deepEqual(samplePath(track, 0.25), { x: 200, y: 150 });
    assert.deepEqual(samplePath(track, 0.75), { x: 400, y: 300 });
  });

  it("returns null for an empty track", () => {
    assert.equal(samplePath([], 1), null);
  });
});

const at = (clicks: ClickEvent[], tS: number) => {
  const state = cursorAt(track, clicks, tS);
  assert.ok(state);
  return state;
};

describe("cursorAt", () => {
  it("holds perfectly still between glides", () => {
    const first = at([], 2);
    for (let f = 1; f < 60; f++) {
      const s = at([], 2 + f / 30);
      assert.equal(s.x, first.x);
      assert.equal(s.y, first.y);
    }
  });

  it("is deterministic", () => {
    const a = at([], 2.5);
    const b = at([], 2.5);
    assert.deepEqual(a, b);
  });

  it("ripples and squashes on mousedown, not on focus beats", () => {
    const clicks = [
      { tMs: 600, x: 300, y: 200, tDownMs: 500 },
      { tMs: 900, x: 400, y: 300 },
    ];
    const during = at(clicks, 0.55);
    assert.ok(during.ripple !== null && during.ripple > 0);
    assert.ok(during.squash < 1, "no squash on mousedown");

    const after = at(clicks, 1.2);
    assert.equal(after.ripple, null);
    assert.equal(after.squash, 1);

    assert.equal(at(clicks, 0.95).ripple, null);
  });

  it("stays pressed for the whole drag and springs back after the drop", () => {
    const drag = [{ tMs: 900, x: 500, y: 400, tDownMs: 200, tUpMs: 900 }];
    assert.ok(at(drag, 0.25).ripple !== null);
    const held = at(drag, 0.8);
    assert.equal(held.squash, CLICK_SQUASH);
    assert.equal(held.ripple, null);
    const springing = at(drag, 0.9 + CLICK_SQUASH_S / 2).squash;
    assert.ok(springing > CLICK_SQUASH && springing < 1);
    assert.equal(at(drag, 0.9 + CLICK_SQUASH_S + 0.01).squash, 1);
  });

  it("returns null when there is no recorded track", () => {
    assert.equal(cursorAt([], [], 1), null);
  });
});

describe("cursorOpacity", () => {
  const typing = [{ tMs: 500, x: 300, y: 200, tDownMs: 450, typeEndMs: 3000 }];

  it("hides the arrow while the app's text caret is up", () => {
    assert.equal(cursorOpacity(typing, 1.5), 0);
    assert.equal(cursorOpacity(typing, 2.9), 0);
  });

  it("is fully visible well outside the typing window", () => {
    assert.equal(cursorOpacity(typing, 0.1), 1);
    assert.equal(cursorOpacity(typing, 4.0), 1);
  });

  it("fades rather than pops at both edges", () => {
    const out = cursorOpacity(typing, 0.5 - CURSOR_FADE_S / 2);
    const back = cursorOpacity(typing, 3.0 + CURSOR_FADE_S / 2);
    for (const v of [out, back]) {
      assert.ok(v > 0 && v < 1, `expected a partial fade, got ${v}`);
    }
  });

  it("leaves plain clicks alone", () => {
    const clicks = [{ tMs: 500, x: 1, y: 1, tDownMs: 450 }];
    assert.equal(cursorOpacity(clicks, 0.5), 1);
  });
});
