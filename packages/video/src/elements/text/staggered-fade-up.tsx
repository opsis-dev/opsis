import { interpolate } from "@opsis/runtime/interpolate.ts";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function StaggeredFadeUp({
  fontFamily,
  text,
  staggerDelay,
  distance,
  fontSize,
  color,
  fontWeight,
  speed,
  background,
  wordDuration,
  letterSpacing,
}: ElementSettings["staggered-fade-up"]) {
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
        background,
      }}
    >
      <span
        style={{
          fontSize,
          fontWeight,
          color,
          letterSpacing: `${letterSpacing}em`,
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        {words.map((word, i) => {
          const local = frame - i * staggerDelay;
          const opacity = interpolate(local, [0, wordDuration], [0, 1], CLAMPED);
          const y = interpolate(local, [0, wordDuration], [distance, 0], CLAMPED);
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
