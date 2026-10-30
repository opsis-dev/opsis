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
import type { MarkLook } from "#video/core/hand-drawn-marks.ts";
import { HandDrawnMark } from "#video/core/hand-drawn.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

const SUBTITLE_FADE = 0.7;

type LineLook = { sizeEm: number; delay: number; style: React.CSSProperties };

const LINES = {
  eyebrow: {
    sizeEm: 0.26,
    delay: 0,
    style: { fontWeight: 700, letterSpacing: "0.18em" },
  },
  title: {
    sizeEm: 1,
    delay: 4,
    style: { fontWeight: 760, lineHeight: 1.02, maxWidth: "100%" },
  },
  subtitle: {
    sizeEm: 0.34,
    delay: 10,
    style: { fontWeight: 450, maxWidth: "80%" },
  },
} satisfies Record<string, LineLook>;

type TitleLineProps = {
  look: LineLook;
  text: React.ReactNode;
  color: string;
  fontSize: number;
  entrance: Entrance;
  stagger: number;
};

export function splitMark(text: string, mark: string): [string, string, string] | null {
  const wanted = mark.trim();
  if (!wanted) return null;
  const at = text.toLowerCase().indexOf(wanted.toLowerCase());
  if (at < 0) return null;
  return [text.slice(0, at), text.slice(at, at + wanted.length), text.slice(at + wanted.length)];
}

type MarkStyle = { thickness: number; markerOpacity: number; seed: number; drawTime: number };

type MarkedTextProps = {
  text: string;
  mark: string;
  look: MarkLook;
  color: string;
  fontSize: number;
  delay: number;
  style: MarkStyle;
};

function MarkedText({ text, mark, look, color, fontSize, delay, style }: MarkedTextProps) {
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
        progress={drawOn(frame / fps, (delay + ENTRANCE_FADE_FRAMES) / fps, style.drawTime)}
        color={look === "marker" ? faded(color, `${style.markerOpacity}%`) : color}
        strokeWidth={fontSize * style.thickness}
        seed={style.seed}
      >
        {marked}
      </HandDrawnMark>
      {after}
    </>
  );
}

function TitleLine({ look, text, color, fontSize, entrance, stagger }: TitleLineProps) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (
    <div
      style={{
        ...entranceStyle(entrance, frame, fps, look.delay * stagger),
        color,
        fontSize: fontSize * look.sizeEm,
        ...look.style,
      }}
    >
      {text}
    </div>
  );
}

export function Title({
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
  sideMargin,
  lineGap,
  eyebrowSize,
  eyebrowUppercase,
  subtitleSize,
  letterSpacing,
  lineStagger,
  markThickness,
  markerOpacity,
  markSeed,
  markDrawTime,
  fadeOut,
  subtitleColor,
}: ElementSettings["title"]) {
  const frame = useCurrentFrame();
  const { durationInFrames, fps } = useVideoConfig();
  const out = exitOpacity(frame, durationInFrames, Math.round(fadeOut * fps));
  const line = { fontSize, entrance, stagger: lineStagger };
  const markStyle = {
    thickness: markThickness,
    markerOpacity,
    seed: markSeed,
    drawTime: markDrawTime,
  };
  return (
    <AbsoluteFill>
      <Backgrounded background={background} />
      <AbsoluteFill
        style={{
          alignItems: align === "center" ? "center" : "flex-start",
          justifyContent: "center",
          padding: `0 ${sideMargin}%`,
          textAlign: align,
          fontFamily: fontOr(fontFamily, "heading"),
          opacity: out,
          gap: fontSize * lineGap,
        }}
      >
        {eyebrow ? (
          <TitleLine
            look={{
              ...LINES.eyebrow,
              sizeEm: eyebrowSize,
              style: {
                ...LINES.eyebrow.style,
                textTransform: eyebrowUppercase ? "uppercase" : "none",
              },
            }}
            text={eyebrow}
            color={accent}
            {...line}
          />
        ) : null}
        <TitleLine
          look={{
            ...LINES.title,
            style: { ...LINES.title.style, fontWeight, letterSpacing: `${letterSpacing}em` },
          }}
          text={
            <MarkedText
              text={title}
              mark={mark}
              look={markLook}
              color={markColor}
              fontSize={fontSize}
              delay={LINES.title.delay * lineStagger}
              style={markStyle}
            />
          }
          color={color}
          {...line}
        />
        {subtitle ? (
          <TitleLine
            look={{
              ...LINES.subtitle,
              sizeEm: subtitleSize,
              style: { ...LINES.subtitle.style, opacity: subtitleColor ? 1 : SUBTITLE_FADE },
            }}
            text={subtitle}
            color={subtitleColor || color}
            {...line}
          />
        ) : null}
      </AbsoluteFill>
    </AbsoluteFill>
  );
}
