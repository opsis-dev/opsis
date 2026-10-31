import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { KeyEvent } from "#video/camera/lib/click-log.ts";
import { useDesignScale } from "#video/camera/lib/design-scale.ts";
import { chordGlyphs, KEYSTROKE_EDGE, keycapAt, keycapEase } from "#video/camera/lib/keycap.ts";
import { useOverlayFrame } from "#video/core/frame.ts";
import type { KeystrokeLook } from "#video/generated/spec.ts";

const KEYCAP = {
  height: 103,
  minWidth: 216,
  padX: 46,
  edgeLift: 3,
  radius: 26,
  glyph: 44,
  glyphGap: 0.28,
  rise: 10,
} as const;

export function KeycapHUD({
  keys,
  timeS,
  look,
}: {
  keys: KeyEvent[];
  timeS: number;
  look: KeystrokeLook;
}) {
  const { position } = look;
  const frameScale = useDesignScale();
  const k = frameScale * look.size;
  const { insets } = useOverlayFrame();
  const active = keycapAt(keys, timeS, look.chordHoldS ?? undefined);
  const edge = ((look.edge ?? KEYSTROKE_EDGE) - KEYCAP.edgeLift) * frameScale;
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
        paddingTop: top ? edge + insets.top : 0,
        paddingBottom: top ? 0 : edge + insets.bottom,
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
          borderRadius: (look.radius ?? KEYCAP.radius) * k,
          backgroundColor: look.keycap,
          color: look.ink,
          fontFamily: look.font || undefined,
          fontSize: KEYCAP.glyph * k,
          gap: KEYCAP.glyphGap * KEYCAP.glyph * k,
          lineHeight: 1,
          opacity,
          transform: `translateY(${rise}px)`,
        }}
      >
        {chordGlyphs(active.chord, look.keyNames)}
      </div>
    </AbsoluteFill>
  );
}
