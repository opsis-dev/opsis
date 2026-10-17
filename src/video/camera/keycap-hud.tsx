import React from "react";
import { AbsoluteFill } from "remotion";
import type { KeyEvent } from "./lib/click-log";
import { chordGlyphs, keycapAt, settle } from "./lib/keycap";
import { useDesignScale } from "./lib/design-scale";
import { useOverlayFrame } from "../core/frame";

const KEYCAP = {
  height: 103,
  minWidth: 216,
  padX: 46,
  edge: 53,
  radius: 26,
  glyph: 44,
  glyphGap: 0.28,
  rise: 10,
  ground: "#0a0a0a",
  ink: "#ffffff",
} as const;

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
  const rise = (1 - appear) * KEYCAP.rise * k * (top ? -1 : 1);

  return (
    <AbsoluteFill
      style={{
        alignItems: "center",
        justifyContent: top ? "flex-start" : "flex-end",
        paddingTop: top ? KEYCAP.edge * k + insets.top : 0,
        paddingBottom: top ? 0 : KEYCAP.edge * k + insets.bottom,
        pointerEvents: "none",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          minWidth: KEYCAP.minWidth * k,
          height: KEYCAP.height * k,
          padding: `0 ${KEYCAP.padX * k}px`,
          borderRadius: KEYCAP.radius * k,
          backgroundColor: KEYCAP.ground,
          color: KEYCAP.ink,
          fontSize: KEYCAP.glyph * k,
          gap: KEYCAP.glyphGap * KEYCAP.glyph * k,
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
