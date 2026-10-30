import { Easing, interpolate } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function SoftBlurIn({
  fontFamily,
  text,
  blur,
  fontSize,
  color,
  fontWeight,
  speed,
  background,
  stagger,
  distance,
  letterSpacing,
}: ElementSettings["soft-blur-in"]) {
  const frame = useAuthoredFrame() * speed;

  const chars = Array.from(text);
  const charDurationFrames = 27;
  const charTravelFrames = 9;

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
        {chars.map((char, i) => {
          const local = frame - i * stagger;
          const easing = Easing.bezier(0.22, 1, 0.36, 1);
          const opacity = interpolate(local, [0, charDurationFrames], [0, 1], {
            ...CLAMPED,
            easing,
          });
          const y = interpolate(local, [0, charTravelFrames], [distance, 0], {
            ...CLAMPED,
            easing,
          });
          const blurAmount = interpolate(local, [0, charDurationFrames], [blur, 0], {
            ...CLAMPED,
            easing,
          });
          return (
            <span
              key={i}
              style={{
                display: "inline-block",
                whiteSpace: "pre",
                backfaceVisibility: "hidden",
                transformOrigin: "50% 55%",
                opacity,
                translate: `0 ${y}px`,
                filter: `blur(${blurAmount}px)`,
              }}
            >
              {char}
            </span>
          );
        })}
      </span>
    </div>
  );
}
