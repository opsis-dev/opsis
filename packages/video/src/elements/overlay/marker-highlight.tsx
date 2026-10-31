import { interpolate } from "@opsis/runtime/interpolate.ts";
import { spring } from "@opsis/runtime/spring.ts";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { interpolateColors } from "#video/lib/ui-kit/color.ts";

export function MarkerHighlight({
  fontFamily,
  before,
  highlight,
  after,
  markerColor,
  baseColor,
  highlightedTextColor,
  fontSize,
  fontWeight,
  speed,
  background,
  markerAt,
  markerHeight,
  x,
  y,
}: ElementSettings["marker-highlight"]) {
  const frame = useAuthoredFrame() * speed;
  const { fps } = useAuthoredConfig();

  const markerScale = spring({
    frame: frame - markerAt,
    fps,
    config: { damping: 14 },
  });

  const textColor = interpolateColors(
    interpolate(markerScale, [0.5, 0.8], [0, 1], CLAMPED),
    [0, 1],
    [baseColor, highlightedTextColor],
  );

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        background,
      }}
    >
      <span
        style={{
          position: "absolute",
          left: `${x * 100}%`,
          top: `${y * 100}%`,
          translate: "-50% -50%",
          whiteSpace: "nowrap",
          fontSize,
          fontWeight,
          color: baseColor,
          letterSpacing: "-0.03em",
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        {before}
        <span style={{ position: "relative", display: "inline-block" }}>
          <span
            aria-hidden
            style={{
              position: "absolute",
              inset: `${(1 - markerHeight) * 100}% -0.1em 0`,
              background: markerColor,
              transformOrigin: "left center",
              scale: `${markerScale} 1`,
              zIndex: 0,
            }}
          />
          <span style={{ position: "relative", zIndex: 1, color: textColor }}>{highlight}</span>
        </span>
        {after}
      </span>
    </div>
  );
}
