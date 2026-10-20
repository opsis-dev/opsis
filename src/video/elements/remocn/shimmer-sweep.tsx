import { interpolate } from "remotion";
import { useAuthoredConfig, useAuthoredFrame } from "@/video/core/authored-clock";

export interface ShimmerSweepProps {
  fontFamily?: string;
  text: string;
  baseColor?: string;
  shineColor?: string;
  fontSize?: number;
  fontWeight?: number;
  speed?: number;
  className?: string;
}

export function ShimmerSweep({
  fontFamily = "var(--font-geist-sans), -apple-system, BlinkMacSystemFont, sans-serif",
  text,
  baseColor = "#3f3f46",
  shineColor = "#fafafa",
  fontSize = 96,
  fontWeight = 700,
  speed = 1,
  className,
}: ShimmerSweepProps) {
  const frame = useAuthoredFrame() * speed;
  const { durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;

  const position = interpolate(
    frame,
    [0, durationInFrames * 0.8],
    [200, -100],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  const textStyle: React.CSSProperties = {
    fontSize,
    fontWeight,
    letterSpacing: "-0.03em",
    fontFamily,
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
        background: "white",
      }}
    >
      <div style={{ position: "relative", display: "inline-block" }}>
        <span style={{ ...textStyle, color: baseColor }}>{text}</span>
        <span
          className={className}
          style={{
            ...textStyle,
            position: "absolute",
            inset: 0,
            color: "transparent",
            backgroundClip: "text",
            WebkitBackgroundClip: "text",
            backgroundImage: `linear-gradient(110deg, transparent 30%, ${shineColor} 50%, transparent 70%)`,
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
