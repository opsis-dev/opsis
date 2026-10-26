import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { faded } from "#video/camera/lib/style.ts";
import { fontOr } from "#video/common/fonts.ts";
import {
  drawOn,
  ENTRANCE_FADE_FRAMES,
  entranceStyle,
  exitOpacity,
  type Entrance,
} from "#video/core/anim.ts";
import { Backgrounded } from "#video/core/backgrounds.tsx";
import { HandDrawnMark, type MarkLook } from "#video/core/hand-drawn.tsx";
import type { CoreElementProps } from "#video/generated/spec.ts";

const TITLE = { padding: "0 9%", gapEm: 0.22 } as const;

type LineLook = { sizeEm: number; delay: number; style: React.CSSProperties };

const MARK = { strokeEm: 0.045, markerShare: "55%", seed: 1 } as const;

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
  text: React.ReactNode;
  color: string;
  fontSize: number;
  entrance: Entrance;
};

export function splitMark(text: string, mark: string): [string, string, string] | null {
  const wanted = mark.trim();
  if (!wanted) return null;
  const at = text.toLowerCase().indexOf(wanted.toLowerCase());
  if (at < 0) return null;
  return [text.slice(0, at), text.slice(at, at + wanted.length), text.slice(at + wanted.length)];
}

type MarkedTextProps = {
  text: string;
  mark: string;
  look: MarkLook;
  color: string;
  fontSize: number;
  delay: number;
};

const MarkedText: React.FC<MarkedTextProps> = ({ text, mark, look, color, fontSize, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const parts = splitMark(text, mark);
  if (!parts) return text;
  const [before, marked, after] = parts;
  return (
    <>
      {before}
      <HandDrawnMark
        look={look}
        progress={drawOn(frame / fps, (delay + ENTRANCE_FADE_FRAMES) / fps)}
        color={look === "marker" ? faded(color, MARK.markerShare) : color}
        strokeWidth={fontSize * MARK.strokeEm}
        seed={MARK.seed}
      >
        {marked}
      </HandDrawnMark>
      {after}
    </>
  );
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
  mark,
  markLook,
  markColor,
  align,
  entrance,
  color,
  accent,
  background,
  fontSize,
  fontWeight,
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
        <TitleLine
          look={{ ...LINES.title, style: { ...LINES.title.style, fontWeight } }}
          text={
            <MarkedText
              text={title}
              mark={mark}
              look={markLook}
              color={markColor}
              fontSize={fontSize}
              delay={LINES.title.delay}
            />
          }
          color={color}
          {...line}
        />
        {subtitle ? (
          <TitleLine look={LINES.subtitle} text={subtitle} color={color} {...line} />
        ) : null}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
