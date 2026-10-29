import type { CSSProperties } from "react";
import { interpolate } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function ShimmerSweep({
  fontFamily,
  text,
  baseColor,
  shineColor,
  fontSize,
  fontWeight,
  speed,
  background,
  sweeps,
  bandAngle,
  bandWidth,
  letterSpacing,
}: ElementSettings["shimmer-sweep"]) {
  const frame = useAuthoredFrame() * speed;
  const { durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;

  const passes = Math.max(1, sweeps);
  const progress =
    interpolate(frame, [0, durationInFrames * 0.8], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }) * passes;
  const pass = progress >= passes ? 1 : progress % 1;
  const position = 200 - 300 * pass;

  const textStyle: CSSProperties = {
    fontSize,
    fontWeight,
    letterSpacing: `${letterSpacing}em`,
    fontFamily: fontOr(fontFamily, "main"),
    margin: 0,
    lineHeight: 1,
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
        <span style={{ ...textStyle, color: baseColor }}>{text}</span>
        <span
          style={{
            ...textStyle,
            position: "absolute",
            inset: 0,
            color: "transparent",
            backgroundClip: "text",
            WebkitBackgroundClip: "text",
            backgroundImage: `linear-gradient(${bandAngle}deg, transparent ${50 - bandWidth / 2}%, ${shineColor} 50%, transparent ${50 + bandWidth / 2}%)`,
            backgroundSize: "200% 100%",
            backgroundPosition: `${position}% 50%`,
          }}
        >
          {text}
        </span>
      </div>
    </div>
  );
}
