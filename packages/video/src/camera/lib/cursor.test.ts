import { describe, expect, it } from "vitest";
import type { ClickEvent, CursorSample } from "./click-log";
import { cursorAt } from "./cursor";

const track: CursorSample[] = [
  { t: 0, x: 100, y: 100 },
  { t: 500, x: 300, y: 200 },
  { t: 1000, x: 500, y: 400 },
];

const at = (clicks: ClickEvent[], tS: number) => {
  const state = cursorAt(track, clicks, tS);
  if (!state) throw new Error(`no cursor at ${tS}s`);
  return state;
};

const pointAt = (tS: number) => {
  const { x, y } = at([], tS);
  return { x, y };
};

describe("cursor path", () => {
  it("holds the ends outside the recorded range", () => {
    expect(pointAt(-5)).toEqual({ x: 100, y: 100 });
    expect(pointAt(99)).toEqual({ x: 500, y: 400 });
  });

  it("interpolates between samples", () => {
    expect(pointAt(0.25)).toEqual({ x: 200, y: 150 });
    expect(pointAt(0.75)).toEqual({ x: 400, y: 300 });
  });
});

describe("cursorAt", () => {
  it("holds perfectly still between glides", () => {
    const first = at([], 2);
    for (let f = 1; f < 60; f++) {
      const s = at([], 2 + f / 30);
      expect(s.x).toBe(first.x);
      expect(s.y).toBe(first.y);
    }
  });

  it("ripples and squashes on mousedown, not on focus beats", () => {
    const clicks = [
      { tMs: 600, x: 300, y: 200, tDownMs: 500 },
      { tMs: 900, x: 400, y: 300 },
    ];
    const during = at(clicks, 0.55);
    expect(during.ripple !== null && during.ripple > 0).toBe(true);
    expect(during.squash, "no squash on mousedown").toBeLessThan(1);

    const after = at(clicks, 1.2);
    expect(after.ripple).toBe(null);
    expect(after.squash).toBe(1);

    expect(at(clicks, 0.95).ripple).toBe(null);
  });

  it("stays pressed for the whole drag and springs back after the drop", () => {
    const drag = [{ tMs: 900, x: 500, y: 400, tDownMs: 200, tUpMs: 900 }];
    expect(at(drag, 0.25).ripple !== null).toBe(true);
    const held = at(drag, 0.8);
    expect(held.squash).toBeLessThan(1);
    expect(at(drag, 0.5).squash).toBe(held.squash);
    expect(held.ripple).toBe(null);
    const springing = at(drag, 0.95).squash;
    expect(springing > held.squash && springing < 1).toBe(true);
    expect(at(drag, 1.5).squash).toBe(1);
  });

  it("returns null when there is no recorded track", () => {
    expect(cursorAt([], [], 1)).toBe(null);
  });
});

describe("cursor opacity", () => {
  const typing = [{ tMs: 500, x: 300, y: 200, tDownMs: 450, typeEndMs: 3000 }];
  const cursorOpacity = (clicks: ClickEvent[], tS: number) => at(clicks, tS).opacity;

  it("hides the arrow while the app's text caret is up", () => {
    expect(cursorOpacity(typing, 1.5)).toBe(0);
    expect(cursorOpacity(typing, 2.9)).toBe(0);
  });

  it("is fully visible well outside the typing window", () => {
    expect(cursorOpacity(typing, 0.1)).toBe(1);
    expect(cursorOpacity(typing, 4.0)).toBe(1);
  });

  it("fades rather than pops at both edges", () => {
    const out = cursorOpacity(typing, 0.45);
    const back = cursorOpacity(typing, 3.05);
    for (const v of [out, back]) {
      expect(v > 0 && v < 1, `expected a partial fade, got ${v}`).toBe(true);
    }
  });

  it("leaves plain clicks alone", () => {
    const clicks = [{ tMs: 500, x: 1, y: 1, tDownMs: 450 }];
    expect(cursorOpacity(clicks, 0.5)).toBe(1);
  });
});

describe("cursor shape", () => {
  const shaped: CursorSample[] = [
    { t: 0, x: 100, y: 100 },
    { t: 400, x: 300, y: 200, c: "pointer" },
    { t: 500, x: 300, y: 200 },
    { t: 600, x: 300, y: 200, c: "default" },
    { t: 1000, x: 500, y: 400, c: "text" },
  ];
  const shapeAt = (tS: number) => cursorAt(shaped, [], tS)?.shape;

  it("shows the arrow until the page asks for another cursor", () => {
    expect(shapeAt(0)).toBe("default");
    expect(shapeAt(0.399)).toBe("default");
  });

  it("keeps the latest shape until the next change", () => {
    expect(shapeAt(0.4)).toBe("pointer");
    expect(shapeAt(0.55)).toBe("pointer");
    expect(shapeAt(0.7)).toBe("default");
    expect(shapeAt(5)).toBe("text");
  });

  it("shows the arrow for takes recorded without shapes", () => {
    expect(cursorAt(track, [], 0.5)?.shape).toBe("default");
  });
});
