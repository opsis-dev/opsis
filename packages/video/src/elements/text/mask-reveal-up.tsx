import { Easing, interpolate } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function MaskRevealUp({
  fontFamily,
  text,
  distance,
  fontSize,
  color,
  fontWeight,
  speed,
  background,
  align,
  lineStagger,
  exitDistance,
  blur: softness,
  lineHeight,
}: ElementSettings["mask-reveal-up"]) {
  const frame = useAuthoredFrame() * speed;
  const { durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;

  const lines = text.split("\n");

  const enterDur = 23;
  const enterTravel = 11;
  const exitDur = 16;
  const exitTravelFrom = 8;
  const enterStagger = lineStagger;
  const exitStagger = 2;

  const enterEasing = Easing.bezier(0.22, 1, 0.36, 1);
  const exitEasing = Easing.bezier(0.64, 0, 0.78, 0);

  const enterEnd = enterDur + (lines.length - 1) * enterStagger;
  const exitStart = Math.max(
    enterEnd,
    durationInFrames - exitDur - (lines.length - 1) * exitStagger,
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
        style={{
          fontSize,
          fontWeight,
          color,
          letterSpacing: "-0.03em",
          lineHeight,
          textAlign: align,
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        {lines.map((line, i) => {
          const enterLocal = frame - i * enterStagger;
          const exitLocal = frame - exitStart - i * exitStagger;

          const enterP = interpolate(enterLocal, [0, enterDur], [0, 1], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: enterEasing,
          });

          const exitP = interpolate(exitLocal, [0, exitDur], [0, 1], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: exitEasing,
          });

          const opacity = enterP * (1 - exitP);

          const yEnter = interpolate(enterLocal, [0, enterTravel], [distance, 0], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: enterEasing,
          });

          const yExit = interpolate(exitLocal, [exitTravelFrom, exitDur], [0, -exitDistance], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: exitEasing,
          });

          const y = yEnter + yExit;

          const blurEnter = interpolate(enterLocal, [0, enterDur], [softness, 0], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: enterEasing,
          });

          const blurExit = interpolate(exitLocal, [0, exitDur], [0, softness], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: exitEasing,
          });

          const blur = blurEnter + blurExit;

          return (
            <span
              key={i}
              style={{
                display: "block",
                transformOrigin: "50% 50%",
                opacity,
                translate: `0 ${y}px`,
                filter: `blur(${blur}px)`,
              }}
            >
              {line}
            </span>
          );
        })}
      </span>
    </div>
  );
}
