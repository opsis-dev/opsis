import React from "react";
import { AbsoluteFill } from "remotion";
import { chordGlyphs, type KeyEvent } from "./lib/click-log";
import { keycapAt, settle } from "./lib/keycap";
import { useDesignScale } from "./lib/design-scale";
import { useOverlayFrame } from "../core/frame";

const KEYCAP_H = 103;
const KEYCAP_MIN_W = 216;
const KEYCAP_PAD_X = 46;
const KEYCAP_EDGE = 53;
const KEYCAP_RADIUS = 26;
const KEYCAP_GLYPH = 44;
const KEYCAP_GROUND = "#0a0a0a";
const KEYCAP_INK = "#ffffff";

export const KeycapHUD: React.FC<{
  keys: KeyEvent[];
  timeS: number;
  position: "top" | "bottom";
}> = ({ keys, timeS, position }) => {
  const k = useDesignScale();
  const { insets } = useOverlayFrame();
  const active = keycapAt(keys, timeS);
  if (!active) return null;

  const appear = 1 - settle(active.inP);
  const leave = 1 - settle(active.outP);
  const opacity = Math.min(appear, leave);
  const top = position === "top";
  const rise = (1 - appear) * 10 * k * (top ? -1 : 1);

  return (
    <AbsoluteFill
      style={{
        alignItems: "center",
        justifyContent: top ? "flex-start" : "flex-end",
        paddingTop: top ? KEYCAP_EDGE * k + insets.top : 0,
        paddingBottom: top ? 0 : KEYCAP_EDGE * k + insets.bottom,
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
