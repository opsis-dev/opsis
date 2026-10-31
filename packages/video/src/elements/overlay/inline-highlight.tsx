import { interpolate } from "@opsis/runtime/interpolate.ts";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { interpolateColors } from "#video/lib/ui-kit/color.ts";

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
    CLAMPED,
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
