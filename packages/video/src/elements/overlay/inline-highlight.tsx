import { interpolate, interpolateColors } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function InlineHighlight({
  fontFamily,
  before,
  highlight,
  after,
  baseColor,
  highlightColor,
  fontSize,
  fontWeight,
  speed,
  background,
  changeStart,
  changeEnd,
  letterSpacing,
}: ElementSettings["inline-highlight"]) {
  const frame = useAuthoredFrame() * speed;
  const { durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;

  const progress = interpolate(
    frame,
    [durationInFrames * changeStart, durationInFrames * changeEnd],
    [0, 1],
    {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    },
  );

  const color = interpolateColors(progress, [0, 1], [baseColor, highlightColor]);

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
      <span
        style={{
          fontSize,
          fontWeight,
          color: baseColor,
          letterSpacing: `${letterSpacing}em`,
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        {before}
        <span style={{ color }}>{highlight}</span>
        {after}
      </span>
    </div>
  );
}
