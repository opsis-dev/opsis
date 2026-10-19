import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import type { CoreElementProps } from "../../shared/elements/core";
import { fontOr } from "../../shared/fonts";
import { entranceStyle, exitOpacity, type Entrance } from "./anim";
import { Backgrounded } from "./backgrounds";

const TITLE = { padding: "0 9%", gapEm: 0.22 } as const;

type LineLook = { sizeEm: number; delay: number; style: React.CSSProperties };

const LINES = {
  eyebrow: {
    sizeEm: 0.26,
    delay: 0,
    style: { fontWeight: 700, letterSpacing: "0.18em", textTransform: "uppercase" },
  },
  title: {
    sizeEm: 1,
    delay: 4,
    style: { fontWeight: 760, letterSpacing: "-0.035em", lineHeight: 1.02, maxWidth: "100%" },
  },
  subtitle: {
    sizeEm: 0.34,
    delay: 10,
    style: { fontWeight: 450, opacity: 0.7, maxWidth: "80%" },
  },
} satisfies Record<string, LineLook>;

type TitleLineProps = {
  look: LineLook;
  text: string;
  color: string;
  fontSize: number;
  entrance: Entrance;
};

const TitleLine: React.FC<TitleLineProps> = ({ look, text, color, fontSize, entrance }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (
    <div
      style={{
        ...entranceStyle(entrance, frame, fps, look.delay),
        color,
        fontSize: fontSize * look.sizeEm,
        ...look.style,
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
  const { durationInFrames } = useVideoConfig();
  const out = exitOpacity(frame, durationInFrames);
  const line = { fontSize, entrance };
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
          <TitleLine look={LINES.eyebrow} text={eyebrow} color={accent} {...line} />
        ) : null}
        <TitleLine look={LINES.title} text={title} color={color} {...line} />
        {subtitle ? (
          <TitleLine look={LINES.subtitle} text={subtitle} color={color} {...line} />
        ) : null}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
