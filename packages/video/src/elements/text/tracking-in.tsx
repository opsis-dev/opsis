import { interpolate, spring } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function TrackingIn({
  fontFamily,
  text,
  startTracking,
  startBlur,
  fontSize,
  color,
  fontWeight,
  speed,
  background,
  endTracking,
  fadeIn,
}: ElementSettings["tracking-in"]) {
  const frame = useAuthoredFrame() * speed;
  const { fps } = useAuthoredConfig();

  const t = spring({
    frame,
    fps,
    config: { damping: 18, stiffness: 90 },
  });

  const letterSpacing = `${interpolate(t, [0, 1], [startTracking, endTracking])}em`;
  const blurAmount = interpolate(t, [0, 1], [startBlur, 0]);
  const opacity = interpolate(frame, [0, fadeIn], [0, 1], CLAMPED);

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
          letterSpacing,
          opacity,
          filter: `blur(${blurAmount}px)`,
          fontFamily: fontOr(fontFamily, "main"),
          whiteSpace: "nowrap",
        }}
      >
        {text}
      </span>
    </div>
  );
}
