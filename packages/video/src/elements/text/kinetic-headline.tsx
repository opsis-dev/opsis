import React, { useMemo } from "react";
import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED, exitOpacity } from "#video/core/anim.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";
import type { CoreElementProps } from "#video/generated/spec.ts";

const WORD_SPRING = { damping: 18, stiffness: 160 };
const EMPHASIS_SWEEP_FRAMES = [8, 22];
const WORD_FADE_FRAMES = 6;

const KINETIC = {
  padding: "0 8%",
  columnGapEm: 0.28,
  rowGapEm: 0.08,
  weight: 780,
  letterSpacing: "-0.04em",
  lineHeight: 1.05,
  riseEm: 0.5,
  tiltDeg: 4,
  underlineBottom: "0.02em",
  underlineHeight: "0.09em",
  underlineRadius: 99,
} as const;

type EmphasisedWord = { word: string; em: boolean };

export const emphasisedWords = (text: string): EmphasisedWord[] =>
  text
    .split(/\s+/)
    .filter(Boolean)
    .map((w) => ({
      word: w.replace(/^\*|\*$/g, ""),
      em: w.startsWith("*") || w.endsWith("*"),
    }));

export const KineticHeadline: React.FC<CoreElementProps["kinetic-headline"]> = ({
  text,
  color,
  emphasisColor,
  background,
  fontSize,
  wordStagger,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const words = useMemo(() => emphasisedWords(text), [text]);
  return (
    <AbsoluteFill>
      <Backgrounded background={background} />
      <AbsoluteFill
        style={{
          alignItems: "center",
          justifyContent: "center",
          padding: KINETIC.padding,
          opacity: exitOpacity(frame, durationInFrames),
        }}
      >
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            justifyContent: "center",
            columnGap: fontSize * KINETIC.columnGapEm,
            rowGap: fontSize * KINETIC.rowGapEm,
            fontFamily: fontOr(fontFamily, "heading"),
            fontSize,
            fontWeight: KINETIC.weight,
            letterSpacing: KINETIC.letterSpacing,
            lineHeight: KINETIC.lineHeight,
          }}
        >
          {words.map(({ word, em }, i) => (
            <KineticWord
              key={i}
              word={word}
              em={em}
              frame={frame - i * wordStagger}
              fps={fps}
              color={em ? emphasisColor : color}
              emphasisColor={emphasisColor}
            />
          ))}
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

const KineticWord: React.FC<
  EmphasisedWord & { frame: number; fps: number; color: string; emphasisColor: string }
> = ({ word, em, frame, fps, color, emphasisColor }) => {
  const s = spring({ frame, fps, config: WORD_SPRING });
  const sweep = interpolate(frame, EMPHASIS_SWEEP_FRAMES, [0, 1], {
    ...CLAMPED,
    easing: Easing.out(Easing.cubic),
  });
  return (
    <span
      style={{
        position: "relative",
        display: "inline-block",
        color,
        opacity: interpolate(frame, [0, WORD_FADE_FRAMES], [0, 1], CLAMPED),
        transform: `translateY(${(1 - s) * KINETIC.riseEm}em) rotate(${(1 - s) * KINETIC.tiltDeg}deg)`,
      }}
    >
      {word}
      {em ? (
        <span
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            bottom: KINETIC.underlineBottom,
            height: KINETIC.underlineHeight,
            background: emphasisColor,
            transformOrigin: "0 50%",
            transform: `scaleX(${sweep})`,
            borderRadius: KINETIC.underlineRadius,
          }}
        />
      ) : null}
    </span>
  );
};
