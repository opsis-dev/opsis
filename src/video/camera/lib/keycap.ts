import { Easing } from "remotion";
import type { KeyEvent } from "./click-log";
import { clamp01 } from "../../../shared/math";

const KEYCAP_BEZIER = Easing.bezier(0.15, 0.9, 0.75, 0.95);

export const KEYCAP_HOLD_S = 1.1;
const KEYCAP_IN_S = 0.13;
const KEYCAP_OUT_S = 0.17;

export const settle = (u: number): number => 1 - KEYCAP_BEZIER(clamp01(u));

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
