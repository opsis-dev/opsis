import { describe, expect, it } from "vitest";
import { chordKeys } from "@/shared/format";
import { chordFromEvent } from "./keys";

const key = (
  k: string,
  mods: Partial<Record<"ctrlKey" | "metaKey" | "altKey" | "shiftKey", boolean>> = {},
) =>
  chordFromEvent({
    key: k,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    ...mods,
  });

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

  it.each([
    ["Control+K", ["Control", "K"]],
    ["Shift++", ["Shift", "+"]],
    ["+", ["+"]],
  ])("splits %j into its keys", (chord, keys) => {
    expect(chordKeys(chord)).toEqual(keys);
  });
});
