import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function LineByLineSlide({
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
  exit,
  letterSpacing,
  lineHeight,
}: ElementSettings["line-by-line-slide"]) {
  const frame = useAuthoredFrame() * speed;
  const { durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;

  const lines = text.split("\n");

  const enterDur = 27;
  const enterTravel = 14;
  const exitDur = 18;
  const exitTravelFrom = 8;
  const enterStagger = lineStagger;
  const exitStagger = 2;

  const enterEasing = Easing.bezier(0.22, 1, 0.36, 1);
  const exitEasing = Easing.bezier(0.64, 0, 0.78, 0);

  const enterEnd = enterDur + (lines.length - 1) * enterStagger;
  const exitStart = exit
    ? Math.max(enterEnd, durationInFrames - exitDur - (lines.length - 1) * exitStagger)
    : Number.POSITIVE_INFINITY;

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
          lineHeight,
          textAlign: align,
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        {lines.map((line, i) => {
          const enterLocal = frame - i * enterStagger;
          const exitLocal = frame - exitStart - i * exitStagger;

          const enterP = interpolate(enterLocal, [0, enterDur], [0, 1], {
            ...CLAMPED,
            easing: enterEasing,
          });

          const exitP = interpolate(exitLocal, [0, exitDur], [0, 1], {
            ...CLAMPED,
            easing: exitEasing,
          });

          const opacity = enterP * (1 - exitP);

          const xEnter = interpolate(enterLocal, [0, enterTravel], [-distance, 0], {
            ...CLAMPED,
            easing: enterEasing,
          });

          const xExit = interpolate(exitLocal, [exitTravelFrom, exitDur], [0, distance], {
            ...CLAMPED,
            easing: exitEasing,
          });

          const x = xEnter + xExit;

          return (
            <span
              key={i}
              style={{
                display: "block",
                transformOrigin: "0% 50%",
                opacity,
                translate: `${x}px`,
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
