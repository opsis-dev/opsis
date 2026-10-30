import { useMemo } from "react";
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
import type { ElementSettings } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

const WORD_SPRING = { damping: 18, stiffness: 160 };
const EMPHASIS_SWEEP_FRAMES = [8, 22];
const WORD_FADE_FRAMES = 6;

const KINETIC = {
  rowGapEm: 0.08,
  underlineBottom: "0.02em",
  underlineHeight: "0.09em",
  underlineRadius: 99,
} as const;

type EmphasisedWord = { word: string; em: boolean };

export function emphasisedWords(text: string): EmphasisedWord[] {
  return text
    .split(/\s+/)
    .filter(Boolean)
    .map((w) => ({
      word: w.replace(/^\*|\*$/g, ""),
      em: w.startsWith("*") || w.endsWith("*"),
    }));
}

export function KineticHeadline({
  text,
  color,
  emphasisColor,
  background,
  fontSize,
  wordStagger,
  fontFamily,
  fontWeight,
  letterSpacing,
  lineHeight,
  sideMargin,
  wordGap,
  rise,
  tilt,
  underlineEmphasis,
  fadeOutInFrames,
}: ElementSettings["kinetic-headline"]) {
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
          padding: `0 ${sideMargin}%`,
          opacity: exitOpacity(frame, durationInFrames, fadeOutInFrames),
        }}
      >
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            justifyContent: "center",
            columnGap: fontSize * wordGap,
            rowGap: fontSize * KINETIC.rowGapEm,
            fontFamily: fontOr(fontFamily, "heading"),
            fontSize,
            fontWeight,
            letterSpacing: `${letterSpacing}em`,
            lineHeight,
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
              rise={rise}
              tilt={tilt}
              underline={underlineEmphasis}
            />
          ))}
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}

function KineticWord({
  word,
  em,
  frame,
  fps,
  color,
  emphasisColor,
  rise,
  tilt,
  underline,
}: EmphasisedWord & {
  frame: number;
  fps: number;
  color: string;
  emphasisColor: string;
  rise: number;
  tilt: number;
  underline: boolean;
}) {
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
        transform: `translateY(${(1 - s) * rise}em) rotate(${(1 - s) * tilt}deg)`,
      }}
    >
      {word}
      {em && underline ? (
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
}
