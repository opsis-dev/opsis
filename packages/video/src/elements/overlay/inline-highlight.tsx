import {
  interpolate,
  interpolateColors,
} from "remotion";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";

export interface InlineHighlightProps {
  background?: string;
  fontFamily?: string;
  before: string;
  highlight: string;
  after?: string;
  baseColor?: string;
  highlightColor?: string;
  fontSize?: number;
  fontWeight?: number;
  speed?: number;
  className?: string;
}

export function InlineHighlight({
  fontFamily = "var(--font-geist-sans), -apple-system, BlinkMacSystemFont, sans-serif",
  before,
  highlight,
  after = "",
  baseColor = "#171717",
  highlightColor = "#ff5e3a",
  fontSize = 48,
  fontWeight = 600,
  speed = 1,
  className,
  background = "transparent",
}: InlineHighlightProps) {
  const frame = useAuthoredFrame() * speed;
  const { durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;

  const progress = interpolate(
    frame,
    [durationInFrames * 0.2, durationInFrames * 0.7],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  const color = interpolateColors(
    progress,
    [0, 1],
    [baseColor, highlightColor],
  );

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
        className={className}
        style={{
          fontSize,
          fontWeight,
          color: baseColor,
          letterSpacing: "-0.03em",
          fontFamily,
        }}
      >
        {before}
        <span style={{ color }}>{highlight}</span>
        {after}
      </span>
    </div>
  );
}
