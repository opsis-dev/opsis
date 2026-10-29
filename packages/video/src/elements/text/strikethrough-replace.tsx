import type { CSSProperties } from "react";
import { interpolate } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function StrikethroughReplace({
  fontFamily,
  from,
  to,
  lineColor,
  fontSize,
  color,
  fontWeight,
  speed,
  background,
  strikeEnd,
  swapLength,
  letterSpacing,
  thickness,
  toColor,
}: ElementSettings["strikethrough-replace"]) {
  const frame = useAuthoredFrame() * speed;
  const { durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;

  const struck = durationInFrames * strikeEnd;
  const fadeStart = durationInFrames * strikeEnd;
  const fadeEnd = durationInFrames * (strikeEnd + swapLength);

  const linePct = interpolate(frame, [0, struck], [0, 100], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const fromOpacity = interpolate(frame, [fadeStart, fadeEnd], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const toOpacity = interpolate(frame, [fadeStart, fadeEnd], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const toY = interpolate(frame, [fadeStart, fadeEnd], [8, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const textStyle: CSSProperties = {
    fontSize,
    fontWeight,
    color,
    letterSpacing: `${letterSpacing}em`,
    fontFamily: fontOr(fontFamily, "main"),
    whiteSpace: "nowrap",
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
      <div
        style={{
          position: "relative",
          display: "inline-flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <span
          style={{
            ...textStyle,
            position: "absolute",
            opacity: fromOpacity,
          }}
        >
          {from}
          <span
            aria-hidden
            style={{
              position: "absolute",
              left: 0,
              top: "50%",
              height: thickness ?? Math.max(2, Math.round(fontSize * 0.08)),
              width: `${linePct}%`,
              background: lineColor,
              translate: "0 -50%",
              borderRadius: 2,
            }}
          />
        </span>

        <span
          style={{
            ...textStyle,
            color: toColor || color,
            opacity: toOpacity,
            translate: `0 ${toY}px`,
          }}
        >
          {to}
        </span>
      </div>
    </div>
  );
}
