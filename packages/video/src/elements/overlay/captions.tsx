import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { spring } from "@opsis/runtime/spring.ts";
import { useMemo } from "react";
import { fontOr } from "#video/common/fonts.ts";
import { framesToMs, msToFrames } from "#video/common/time.ts";
import { JUSTIFY_AT, overlayPadding, useOverlayFrame, verticalInsets } from "#video/core/frame.ts";
import type { ElementSettings, CaptionsValue } from "#video/generated/spec.ts";

const CAPTIONS = {
  wordGapEm: 0.28,
  shadow: "0 4px 24px rgba(0,0,0,0.55)",
  boxPadShare: 10 / 22,
  popFrom: 0.85,
  popGrowth: 0.15,
} as const;

const CAPTION_POP_SPRING = { damping: 14, stiffness: 220 };

type Caption = CaptionsValue["captions"][number];

export function groupCaptions(captions: Caption[], wordsPerLine: number): Caption[][] {
  const lines: Caption[][] = [];
  let line: Caption[] = [];
  for (const caption of captions) {
    line.push(caption);
    if (line.length >= wordsPerLine || /[.!?]$/.test(caption.text)) {
      lines.push(line);
      line = [];
    }
  }
  if (line.length) lines.push(line);
  return lines;
}

type CaptionStyle = ElementSettings["captions"]["style"];

function isLit(style: CaptionStyle, spoken: boolean, active: boolean): boolean {
  switch (style) {
    case "karaoke":
      return spoken;
    case "word-by-word":
      return active;
    case "line":
      return false;
  }
}

function CaptionWord({
  word,
  style,
  tMs,
  color,
  highlightColor,
  unspoken,
}: {
  word: Caption;
  style: CaptionStyle;
  tMs: number;
  color: string;
  highlightColor: string;
  unspoken: number;
}) {
  const frame = useFrame();
  const { fps } = useFilm();
  const active = tMs >= word.startMs && tMs < word.endMs;
  const spoken = tMs >= word.startMs;
  if (style === "word-by-word" && !spoken) return null;
  const pop = spring({
    frame: frame - msToFrames(word.startMs, fps, 0),
    fps,
    config: CAPTION_POP_SPRING,
  });
  return (
    <span
      style={{
        color: isLit(style, spoken, active) ? highlightColor : color,
        display: "inline-block",
        transform:
          style === "word-by-word"
            ? `scale(${CAPTIONS.popFrom + pop * CAPTIONS.popGrowth})`
            : undefined,
        opacity: style === "karaoke" && !spoken ? unspoken : 1,
      }}
    >
      {word.text}
    </span>
  );
}

export function Captions({
  captions,
  style,
  position,
  fontSize,
  color,
  highlightColor,
  panel,
  fontWeight,
  fontFamily,
  wordsPerLine,
  lingerMs,
  edgeOffset,
  sideMargin,
  panelPadding,
  panelRadius,
  textShadow,
  unspokenOpacity,
}: ElementSettings["captions"] & { captions: Caption[] }) {
  const frame = useFrame();
  const { fps } = useFilm();
  const { k, insets } = useOverlayFrame();
  const tMs = framesToMs(frame, fps);
  const lines = useMemo(() => groupCaptions(captions, wordsPerLine), [captions, wordsPerLine]);
  const line = lines.find((l) => tMs >= l[0].startMs && tMs <= l[l.length - 1].endMs + lingerMs);
  if (!line) return null;
  const boxPadX = panelPadding;
  const boxPadY = panelPadding * CAPTIONS.boxPadShare;
  return (
    <AbsoluteFill
      style={{
        justifyContent: JUSTIFY_AT[position],
        alignItems: "center",
        ...overlayPadding(edgeOffset * k, verticalInsets(insets)),
        paddingLeft: `${sideMargin}%`,
        paddingRight: `${sideMargin}%`,
      }}
    >
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          justifyContent: "center",
          gap: `0 ${fontSize * CAPTIONS.wordGapEm * k}px`,
          fontFamily: fontOr(fontFamily, "main"),
          fontSize: fontSize * k,
          fontWeight,
          textShadow: textShadow ? CAPTIONS.shadow : undefined,
          padding: `${boxPadY * k}px ${boxPadX * k}px`,
          borderRadius: panelRadius * k,
          background: panel,
        }}
      >
        {line.map((word, i) => (
          <CaptionWord
            key={i}
            word={word}
            style={style}
            tMs={tMs}
            color={color}
            highlightColor={highlightColor}
            unspoken={unspokenOpacity / 100}
          />
        ))}
      </div>
    </AbsoluteFill>
  );
}
