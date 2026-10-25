import { interpolate } from "remotion";
import { useAuthoredFrame } from "../../core/authored-clock";

export interface StaggeredFadeUpProps {
  fontFamily?: string;
  text: string;
  staggerDelay?: number;
  distance?: number;
  fontSize?: number;
  color?: string;
  fontWeight?: number;
  speed?: number;
  className?: string;
}

export function StaggeredFadeUp({
  fontFamily = "var(--font-geist-sans), -apple-system, BlinkMacSystemFont, sans-serif",
  text,
  staggerDelay = 4,
  distance = 20,
  fontSize = 72,
  color = "#171717",
  fontWeight = 600,
  speed = 1,
  className,
}: StaggeredFadeUpProps) {
  const frame = useAuthoredFrame() * speed;

  const words = text.split(" ");

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
      <span
        className={className}
        style={{
          fontSize,
          fontWeight,
          color,
          letterSpacing: "-0.03em",
          fontFamily,
        }}
      >
        {words.map((word, i) => {
          const local = frame - i * staggerDelay;
          const opacity = interpolate(local, [0, 12], [0, 1], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
          });
          const y = interpolate(local, [0, 12], [distance, 0], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
          });
          return (
            <span
              key={i}
              style={{
                display: "inline-block",
                marginRight: "0.25em",
                opacity,
                translate: `0 ${y}px`,
              }}
            >
              {word}
            </span>
          );
        })}
      </span>
    </div>
  );
}
