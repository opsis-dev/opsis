import { Easing } from "@opsis/runtime/easing.ts";
import type { KeyEvent } from "#video/camera/lib/click-log.ts";
import { chordKeys } from "#video/common/chord.ts";
import { clamp01 } from "#video/common/math.ts";
import type { KeyNames } from "#video/generated/spec.ts";

const KEYCAP_BEZIER = Easing.bezier(0.15, 0.9, 0.75, 0.95);

const KEYCAP_HOLD_S = 1.1;
export const KEYSTROKE_EDGE = 56;
const KEYCAP_IN_S = 0.13;
const KEYCAP_OUT_S = 0.17;

const KEY_GLYPHS = new Map([
  ["alt", "⌥"],
  ["option", "⌥"],
  ["meta", "⌘"],
  ["cmd", "⌘"],
  ["command", "⌘"],
  ["control", "⌃"],
  ["ctrl", "⌃"],
  ["shift", "⇧"],
  ["enter", "⏎"],
  ["return", "⏎"],
  ["tab", "⇥"],
  ["escape", "esc"],
  ["esc", "esc"],
  ["backspace", "⌫"],
  ["delete", "⌦"],
  ["arrowup", "↑"],
  ["arrowdown", "↓"],
  ["arrowleft", "←"],
  ["arrowright", "→"],
  ["space", "space"],
]);

const KEY_WORDS = new Map([
  ["alt", "Alt"],
  ["option", "Option"],
  ["meta", "Cmd"],
  ["cmd", "Cmd"],
  ["command", "Cmd"],
  ["control", "Ctrl"],
  ["ctrl", "Ctrl"],
  ["shift", "Shift"],
  ["enter", "Enter"],
  ["return", "Return"],
  ["tab", "Tab"],
  ["escape", "Esc"],
  ["esc", "Esc"],
  ["backspace", "Backspace"],
  ["delete", "Delete"],
  ["arrowup", "↑"],
  ["arrowdown", "↓"],
  ["arrowleft", "←"],
  ["arrowright", "→"],
  ["space", "Space"],
]);

export function chordGlyphs(chord: string, names: KeyNames = "symbols"): string {
  const table = names === "words" ? KEY_WORDS : KEY_GLYPHS;
  const keys = chordKeys(chord).map((key) => table.get(key.toLowerCase()) ?? key.toUpperCase());
  return keys.join(names === "words" ? " + " : "");
}

export function keycapEase(u: number): number {
  return KEYCAP_BEZIER(clamp01(u));
}

export function keycapAt(
  keys: KeyEvent[],
  timeS: number,
  holdS = KEYCAP_HOLD_S,
): { chord: string; inP: number; outP: number } | null {
  for (let i = keys.length - 1; i >= 0; i--) {
    const key = keys[i];
    const startS = key.tMs / 1000;
    const endS = startS + holdS;
    if (timeS < startS || timeS >= endS) continue;
    return {
      chord: key.chord,
      inP: clamp01((timeS - startS) / KEYCAP_IN_S),
      outP: clamp01((endS - timeS) / KEYCAP_OUT_S),
    };
  }
  return null;
}

const KEY_ALIASES = new Map([
  ["esc", "escape"],
  ["return", "enter"],
  ["cmd", "meta"],
  ["command", "meta"],
  ["ctrl", "control"],
  ["option", "alt"],
  ["del", "delete"],
  ["up", "arrowup"],
  ["down", "arrowdown"],
  ["left", "arrowleft"],
  ["right", "arrowright"],
]);

function chordId(chord: string): string {
  return chordKeys(chord)
    .map((key) => key.toLowerCase())
    .map((key) => KEY_ALIASES.get(key) ?? key)
    .toSorted()
    .join("+");
}

export function shownKeys(keys: readonly KeyEvent[], leftOut: readonly string[]): KeyEvent[] {
  if (leftOut.length === 0) return [...keys];
  const hidden = new Set(leftOut.map(chordId));
  return keys.filter((key) => !hidden.has(chordId(key.chord)));
}
