import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import type { CoreElementProps } from "../../shared/elements/core";
import { fontOr } from "../../shared/fonts";
import { entranceStyle, exitOpacity, type Entrance } from "./anim";
import { Backgrounded } from "./backgrounds";

const TITLE = {
  padding: "0 9%",
  gapEm: 0.22,
  eyebrow: { sizeEm: 0.26, weight: 700, letterSpacing: "0.18em", delay: 0 },
  title: { weight: 760, letterSpacing: "-0.035em", lineHeight: 1.02, delay: 4 },
  subtitle: { sizeEm: 0.34, weight: 450, opacity: 0.7, maxWidth: "80%", delay: 10 },
} as const;

type TitleLineProps = { text: string; color: string; fontSize: number; entrance: Entrance };

const TitleEyebrow: React.FC<TitleLineProps> = ({ text, color, fontSize, entrance }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { sizeEm, weight, letterSpacing, delay } = TITLE.eyebrow;
  return (
    <div
      style={{
        ...entranceStyle(entrance, frame, fps, delay),
        color,
        fontSize: fontSize * sizeEm,
        fontWeight: weight,
        letterSpacing,
        textTransform: "uppercase",
      }}
    >
      {text}
    </div>
  );
};

const TitleSubtitle: React.FC<TitleLineProps> = ({ text, color, fontSize, entrance }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { sizeEm, weight, opacity, maxWidth, delay } = TITLE.subtitle;
  return (
    <div
      style={{
        ...entranceStyle(entrance, frame, fps, delay),
        color,
        opacity,
        fontSize: fontSize * sizeEm,
        fontWeight: weight,
        maxWidth,
      }}
    >
      {text}
    </div>
  );
};

export const Title: React.FC<CoreElementProps["core/title"]> = ({
  eyebrow,
  title,
  subtitle,
  align,
  entrance,
  color,
  accent,
  background,
  fontSize,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const out = exitOpacity(frame, durationInFrames);
  return (
    <AbsoluteFill>
      <Backgrounded background={background} />
      <AbsoluteFill
        style={{
          alignItems: align === "center" ? "center" : "flex-start",
          justifyContent: "center",
          padding: TITLE.padding,
          textAlign: align,
          fontFamily: fontOr(fontFamily, "heading"),
          opacity: out,
          gap: fontSize * TITLE.gapEm,
        }}
      >
        {eyebrow ? (
          <TitleEyebrow text={eyebrow} color={accent} fontSize={fontSize} entrance={entrance} />
        ) : null}
        <div
          style={{
            ...entranceStyle(entrance, frame, fps, TITLE.title.delay),
            color,
            fontSize,
            fontWeight: TITLE.title.weight,
            letterSpacing: TITLE.title.letterSpacing,
            lineHeight: TITLE.title.lineHeight,
            maxWidth: "100%",
          }}
        >
          {title}
        </div>
        {subtitle ? (
          <TitleSubtitle text={subtitle} color={color} fontSize={fontSize} entrance={entrance} />
        ) : null}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
