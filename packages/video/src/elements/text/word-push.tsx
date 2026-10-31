import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { useMemo } from "react";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

const REVEAL_FRAMES = 2;
const CHAR_WIDTH_EM = 0.5;
const SPACE_WIDTH_EM = 0.22;

function measureWords(
  words: string[],
  fontSize: number,
  fontWeight: string,
  fontFamily: string,
  letterSpacing: number,
): number[] {
  function guess() {
    return words.map((word) => word.length * CHAR_WIDTH_EM * fontSize);
  }
  if (typeof document === "undefined") return guess();
  const ctx = document.createElement("canvas").getContext("2d");
  if (!ctx) return guess();
  ctx.font = `${fontWeight} ${fontSize}px ${fontFamily}`;
  ctx.letterSpacing = `${letterSpacing * fontSize}px`;
  return words.map((word) => ctx.measureText(word).width);
}

function revealFrame(index: number, wordGap: number, accel: number): number {
  if (accel === 1) return index * wordGap;
  return (wordGap * (1 - accel ** index)) / (1 - accel);
}

export function wordPushLength(
  text: string,
  wordGap: number,
  accel: number,
  pushTime: number,
): number {
  const words = text.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return 0;
  return Math.ceil(revealFrame(words.length - 1, wordGap, accel)) + pushTime;
}

const pushEasing = Easing.out(Easing.cubic);
const driveEasing = Easing.bezier(0.15, 0.7, 0.25, 1);

export function WordPush({
  fontFamily,
  text,
  fontSize,
  color,
  fontWeight,
  wordGap,
  accel,
  speed,
  background,
  pushTime,
  driveDistance,
  letterSpacing,
}: ElementSettings["word-push"]) {
  const face = fontOr(fontFamily, "main");
  const frame = useAuthoredFrame() * speed;

  const words = text.trim().split(/\s+/).filter(Boolean);
  const widths = useMemo(
    () =>
      measureWords(
        text.trim().split(/\s+/).filter(Boolean),
        fontSize,
        fontWeight,
        face,
        letterSpacing,
      ),
    [text, fontSize, fontWeight, face, letterSpacing],
  );

  let lineX = interpolate(frame, [0, pushTime], [-driveDistance * fontSize, 0], {
    ...CLAMPED,
    easing: driveEasing,
  });
  for (let j = 1; j < words.length; j++) {
    const wordWidth = widths[j] + SPACE_WIDTH_EM * fontSize;
    const revealAt = revealFrame(j, wordGap, accel);
    const pushed = interpolate(frame, [revealAt + 1, revealAt + 1 + pushTime], [0, 1], {
      ...CLAMPED,
      easing: pushEasing,
    });
    lineX += (wordWidth / 2) * (1 - pushed);
  }

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        whiteSpace: "nowrap",
        translate: `${lineX}px 0`,
        background,
      }}
    >
      <span
        style={{
          fontSize,
          fontWeight,
          color,
          letterSpacing: `${letterSpacing}em`,
          fontFamily: face,
        }}
      >
        {words.map((word, j) => {
          const revealAt = revealFrame(j, wordGap, accel);

          const opacity = interpolate(frame, [revealAt, revealAt + REVEAL_FRAMES], [0, 1], CLAMPED);

          const driven =
            j === 0
              ? 1
              : interpolate(frame, [revealAt, revealAt + pushTime], [0, 1], {
                  ...CLAMPED,
                  easing: driveEasing,
                });

          return (
            <span
              key={j}
              style={{
                display: "inline-block",
                whiteSpace: "pre",
                marginRight: j < words.length - 1 ? "0.22em" : undefined,
                opacity,
                translate: `${driveDistance * fontSize * (1 - driven)}px 0`,
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
