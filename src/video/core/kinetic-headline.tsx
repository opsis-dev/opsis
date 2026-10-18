import React, { useMemo } from "react";
import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import type { CoreElementProps } from "../../shared/elements/core";
import { fontOr } from "../../shared/fonts";
import { CLAMPED, exitOpacity } from "./anim";
import { Backgrounded } from "./backgrounds";

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
} as const;

export const KineticHeadline: React.FC<CoreElementProps["core/kinetic-headline"]> = ({
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

type EmphasisedWord = { word: string; em: boolean };

export const emphasisedWords = (text: string): EmphasisedWord[] =>
  text
    .split(/\s+/)
    .filter(Boolean)
    .map((w) => ({
      word: w.replace(/^\*|\*$/g, ""),
      em: w.startsWith("*") || w.endsWith("*"),
    }));

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
        transform: `translateY(${(1 - s) * 0.5}em) rotate(${(1 - s) * 4}deg)`,
      }}
    >
      {word}
      {em ? (
        <span
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            bottom: "0.02em",
            height: "0.09em",
            background: emphasisColor,
            transformOrigin: "0 50%",
            transform: `scaleX(${sweep})`,
            borderRadius: 99,
          }}
        />
      ) : null}
    </span>
  );
};
