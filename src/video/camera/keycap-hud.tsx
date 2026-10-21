import React from "react";
import { AbsoluteFill } from "remotion";
import type { KeystrokeLook } from "../../shared/schema/recording";
import { useOverlayFrame } from "../core/frame";
import type { KeyEvent } from "./lib/click-log";
import { useDesignScale } from "./lib/design-scale";
import { chordGlyphs, keycapAt, keycapEase } from "./lib/keycap";

const KEYCAP = {
  height: 103,
  minWidth: 216,
  padX: 46,
  edge: 53,
  radius: 26,
  glyph: 44,
  glyphGap: 0.28,
  rise: 10,
} as const;

export const KeycapHUD: React.FC<{
  keys: KeyEvent[];
  timeS: number;
  look: KeystrokeLook;
}> = ({ keys, timeS, look }) => {
  const { position } = look;
  const frameScale = useDesignScale();
  const k = frameScale * look.size;
  const { insets } = useOverlayFrame();
  const active = keycapAt(keys, timeS);
  if (!active) return null;

  const appear = keycapEase(active.inP);
  const opacity = Math.min(appear, keycapEase(active.outP));
  const top = position === "top";
  const rise = (1 - appear) * KEYCAP.rise * k * (top ? -1 : 1);

  return (
    <AbsoluteFill
      style={{
        alignItems: "center",
        justifyContent: top ? "flex-start" : "flex-end",
        paddingTop: top ? KEYCAP.edge * frameScale + insets.top : 0,
        paddingBottom: top ? 0 : KEYCAP.edge * frameScale + insets.bottom,
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
          backgroundColor: look.keycap,
          color: look.ink,
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
