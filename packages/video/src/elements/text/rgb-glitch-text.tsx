import { random } from "@opsis/runtime/random.ts";
import type { CSSProperties } from "react";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function RGBGlitchText({
  fontFamily,
  text,
  fontSize,
  color,
  fontWeight,
  glitchAt,
  glitchDuration,
  intensity,
  seed,
  glitchColors,
  blend,
  speed,
  background,
  letterSpacing,
}: ElementSettings["rgb-glitch-text"]) {
  const frame = useAuthoredFrame() * speed;

  const isGlitching = frame >= glitchAt && frame < glitchAt + glitchDuration;

  function offset(axisSeed: string, scale: number) {
    return (random(`${seed}-${axisSeed}-${frame}`) * 2 - 1) * scale;
  }

  const rX = isGlitching ? offset("r-x", intensity) : 0;
  const rY = isGlitching ? offset("r-y", intensity * 0.4) : 0;
  const gX = isGlitching ? offset("g-x", intensity) : 0;
  const gY = isGlitching ? offset("g-y", intensity * 0.4) : 0;
  const bX = isGlitching ? offset("b-x", intensity) : 0;
  const bY = isGlitching ? offset("b-y", intensity * 0.4) : 0;

  const copyOpacity = isGlitching ? 1 : 0;

  const baseStyle: CSSProperties = {
    position: "absolute",
    top: 0,
    left: 0,
    fontSize,
    fontWeight,
    letterSpacing: `${letterSpacing}em`,
    fontFamily: fontOr(fontFamily, "main"),
    whiteSpace: "pre",
    mixBlendMode: blend,
  };

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background,
      }}
    >
      <div style={{ position: "relative", display: "inline-block" }}>
        <span
          style={{
            position: "relative",
            fontSize,
            fontWeight,
            color,
            letterSpacing: "-0.03em",
            fontFamily,
            whiteSpace: "pre",
          }}
        >
          {text}
        </span>
        <span
          style={{
            ...baseStyle,
            color: glitchColors[0],
            opacity: copyOpacity,
            translate: `${rX}px ${rY}px`,
          }}
        >
          {text}
        </span>
        <span
          style={{
            ...baseStyle,
            color: glitchColors[1],
            opacity: copyOpacity,
            translate: `${gX}px ${gY}px`,
          }}
        >
          {text}
        </span>
        <span
          style={{
            ...baseStyle,
            color: glitchColors[2],
            opacity: copyOpacity,
            translate: `${bX}px ${bY}px`,
          }}
        >
          {text}
        </span>
      </div>
    </div>
  );
}
