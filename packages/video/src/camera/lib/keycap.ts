import { Easing } from "remotion";
import type { KeyEvent } from "#video/camera/lib/click-log.ts";
import { chordKeys } from "#video/common/chord.ts";
import { clamp01 } from "#video/common/math.ts";

const KEYCAP_BEZIER = Easing.bezier(0.15, 0.9, 0.75, 0.95);

const KEYCAP_HOLD_S = 1.1;
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

export const chordGlyphs = (chord: string): string =>
  chordKeys(chord)
    .map((key) => KEY_GLYPHS.get(key.toLowerCase()) ?? key.toUpperCase())
    .join("");

export const keycapEase = (u: number): number => KEYCAP_BEZIER(clamp01(u));

export function keycapAt(
  keys: KeyEvent[],
  timeS: number,
): { chord: string; inP: number; outP: number } | null {
  for (let i = keys.length - 1; i >= 0; i--) {
    const key = keys[i];
    const startS = key.tMs / 1000;
    const endS = startS + KEYCAP_HOLD_S;
    if (timeS < startS || timeS >= endS) continue;
    return {
      chord: key.chord,
      inP: clamp01((timeS - startS) / KEYCAP_IN_S),
      outP: clamp01((endS - timeS) / KEYCAP_OUT_S),
    };
  }
  return null;
}
