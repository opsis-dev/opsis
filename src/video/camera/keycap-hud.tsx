import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { chordGlyphs, type KeyEvent } from "./lib/click-log";
import { settle } from "./lib/push";
import { useOverlayFrame } from "../core/frame";
import { useDesignScale } from "./window-frame";
import { clamp } from "../../shared/math";

export const KEYCAP_H = 103;
export const KEYCAP_MIN_W = 216;
export const KEYCAP_PAD_X = 46;
export const KEYCAP_BOTTOM = 53;
export const KEYCAP_RADIUS = 26;
export const KEYCAP_GLYPH = 44;
export const KEYCAP_GROUND = "#0a0a0a";
export const KEYCAP_INK = "#ffffff";

export const KEYCAP_HOLD_S = 1.1;
export const KEYCAP_IN_S = 0.13;
export const KEYCAP_OUT_S = 0.17;

export type KeycapHUDProps = {
  keys: KeyEvent[];
  timeS: number;
  holdS?: number;
};

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
    const inP = KEYCAP_IN_S > 0 ? (timeS - startS) / KEYCAP_IN_S : 1;
    const outP = KEYCAP_OUT_S > 0 ? (endS - timeS) / KEYCAP_OUT_S : 1;
    return {
      chord: key.chord,
      inP: clamp(inP, 0, 1),
      outP: clamp(outP, 0, 1),
    };
  }
  return null;
}

export const KeycapHUD: React.FC<KeycapHUDProps> = ({ keys, timeS, holdS = KEYCAP_HOLD_S }) => {
  const k = useDesignScale();
  const { insets } = useOverlayFrame();
  const active = keycapAt(keys, timeS, holdS);
  if (!active) return null;

  const appear = 1 - settle(active.inP);
  const leave = 1 - settle(active.outP);
  const opacity = Math.min(appear, leave);
  const rise = (1 - appear) * 10 * k;

  return (
    <AbsoluteFill
      style={{
        alignItems: "center",
        justifyContent: "flex-end",
        paddingBottom: KEYCAP_BOTTOM * k + insets.bottom,
        pointerEvents: "none",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          minWidth: KEYCAP_MIN_W * k,
          height: KEYCAP_H * k,
          padding: `0 ${KEYCAP_PAD_X * k}px`,
          borderRadius: KEYCAP_RADIUS * k,
          backgroundColor: KEYCAP_GROUND,
          color: KEYCAP_INK,
          fontSize: KEYCAP_GLYPH * k,
          gap: 0.28 * KEYCAP_GLYPH * k,
          lineHeight: 1,
          opacity,
          transform: `translateY(${rise}px)`,
        }}
      >
        {chordGlyphs(active.chord)}
      </div>
    </AbsoluteFill>
  );
};

export const useKeycapTime = (speed: number, offsetMs = 0): number => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (frame / fps) * speed - offsetMs / 1000;
};
