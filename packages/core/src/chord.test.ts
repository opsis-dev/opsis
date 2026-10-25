import { describe, expect, it } from "vitest";
import { chordFromEvent, chordKeys, pressedChord } from "#core/chord";
import { keyEvent } from "#core/test-fixtures";

const key = (...args: Parameters<typeof keyEvent>) => chordFromEvent(keyEvent(...args));

describe("recording a shortcut", () => {
  it.each([
    ["k", { ctrlKey: true }, "Control+K"],
    ["Enter", {}, "Enter"],
    [" ", { metaKey: true, shiftKey: true }, "Meta+Shift+Space"],
    ["Shift", { shiftKey: true }, null],
    ["ArrowUp", { altKey: true }, "Alt+ArrowUp"],
    ["Escape", {}, "Escape"],
  ] as const)("writes %j with %j as %j", (k, mods, chord) => {
    expect(key(k, mods)).toBe(chord);
  });
});

describe("chordKeys", () => {
  it.each([
    ["Control+K", ["Control", "K"]],
    ["Shift++", ["Shift", "+"]],
    ["+", ["+"]],
    ["Alt + Enter", ["Alt", "Enter"]],
  ])("splits %j into its keys", (chord, keys) => {
    expect(chordKeys(chord)).toEqual(keys);
  });

  it("presses a chord typed with spaces", () => {
    expect(pressedChord("Alt + Enter")).toBe("Alt+Enter");
  });
});
