import React, { useMemo } from "react";
import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { CLAMPED, entranceStyle, exitOpacity, type Entrance } from "./anim";
import { Backgrounded } from "./backgrounds";
import { isLandscape, JUSTIFY_AT, overlayPadding, useOverlayFrame, verticalInsets } from "./frame";
import { fontOr } from "../../shared/fonts";
import {
  ACCENT_COLOR,
  CALLOUT_COLOR,
  CAPTION_COLOR,
  HIGHLIGHT_COLOR,
  PANEL_COLOR,
  TEXT_COLOR,
} from "../../shared/elements/colors";
import { framesToMs, msToFrames } from "../../shared/timeline/time";

const MAX_WORDS_PER_LINE = 6;
const CAPTION_LINGER_MS = 250;
const LOWER_THIRD_SPRING = { damping: 200 };
const LOWER_THIRD_EXIT_FRAMES = 14;
const LOWER_THIRD_SLIDE_PX = 120;
const WORD_SPRING = { damping: 18, stiffness: 160 };
const EMPHASIS_SWEEP_FRAMES = [8, 22];
const WORD_FADE_FRAMES = 6;

export const Title: React.FC<{
  eyebrow?: string;
  title: string;
  subtitle?: string;
  align?: "center" | "left";
  entrance?: Entrance;
  color?: string;
  accent?: string;
  background?: string;
  fontSize?: number;
  fontFamily?: string;
}> = ({
  eyebrow,
  title,
  subtitle,
  align = "center",
  entrance = "rise",
  color = TEXT_COLOR,
  accent = ACCENT_COLOR,
  background,
  fontSize = 112,
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
          padding: "0 9%",
          textAlign: align,
          fontFamily: fontOr(fontFamily, "heading"),
          opacity: out,
          gap: fontSize * 0.22,
        }}
      >
        {eyebrow ? (
          <div
            style={{
              ...entranceStyle(entrance, frame, fps),
              color: accent,
              fontSize: fontSize * 0.26,
              fontWeight: 700,
              letterSpacing: "0.18em",
              textTransform: "uppercase",
            }}
          >
            {eyebrow}
          </div>
        ) : null}
        <div
          style={{
            ...entranceStyle(entrance, frame, fps, 4),
            color,
            fontSize,
            fontWeight: 760,
            letterSpacing: "-0.035em",
            lineHeight: 1.02,
            maxWidth: "100%",
          }}
        >
          {title}
        </div>
        {subtitle ? (
          <div
            style={{
              ...entranceStyle(entrance, frame, fps, 10),
              color,
              opacity: 0.7,
              fontSize: fontSize * 0.34,
              fontWeight: 450,
              maxWidth: "80%",
            }}
          >
            {subtitle}
          </div>
        ) : null}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

export const KineticHeadline: React.FC<{
  text: string;
  color?: string;
  emphasisColor?: string;
  background?: string;
  fontSize?: number;
  wordStagger?: number;
  fontFamily?: string;
}> = ({
  text,
  color = TEXT_COLOR,
  emphasisColor = HIGHLIGHT_COLOR,
  background,
  fontSize = 104,
  wordStagger = 4,
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
          padding: "0 8%",
          opacity: exitOpacity(frame, durationInFrames),
        }}
      >
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            justifyContent: "center",
            columnGap: fontSize * 0.28,
            rowGap: fontSize * 0.08,
            fontFamily: fontOr(fontFamily, "heading"),
            fontSize,
            fontWeight: 780,
            letterSpacing: "-0.04em",
            lineHeight: 1.05,
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

export const LowerThird: React.FC<{
  title: string;
  subtitle?: string;
  position?: "bottom-left" | "bottom-right" | "top-left" | "top-right";
  accent?: string;
  color?: string;
  panel?: string;
  fontFamily?: string;
}> = ({
  title,
  subtitle,
  position = "bottom-left",
  accent = ACCENT_COLOR,
  color = TEXT_COLOR,
  panel = PANEL_COLOR,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames, width, height } = useVideoConfig();
  const { k, insets } = useOverlayFrame();
  const enter = spring({ frame, fps, config: LOWER_THIRD_SPRING });
  const exit = spring({
    frame: frame - (durationInFrames - LOWER_THIRD_EXIT_FRAMES),
    fps,
    config: LOWER_THIRD_SPRING,
  });
  const top = position.startsWith("top") || !isLandscape(width, height);
  const right = position.endsWith("right");
  const dir = right ? 1 : -1;
  const x = (1 - enter) * LOWER_THIRD_SLIDE_PX * dir + exit * LOWER_THIRD_SLIDE_PX * dir;
  return (
    <AbsoluteFill
      style={{
        justifyContent: top ? "flex-start" : "flex-end",
        alignItems: right ? "flex-end" : "flex-start",
        ...overlayPadding(80 * k, insets),
      }}
    >
      <div
        style={{
          display: "flex",
          gap: 18 * k,
          alignItems: "stretch",
          transform: `translateX(${x * k}px)`,
          opacity: Math.min(enter, 1 - exit),
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        <div
          style={{
            width: 8 * k,
            borderRadius: 99,
            background: accent,
            transform: `scaleY(${enter})`,
            transformOrigin: "50% 100%",
          }}
        />
        <div
          style={{
            background: panel,
            borderRadius: 18 * k,
            padding: `${18 * k}px ${28 * k}px`,
            backdropFilter: "blur(12px)",
          }}
        >
          <div style={{ color, fontSize: 44 * k, fontWeight: 720, letterSpacing: "-0.02em" }}>
            {title}
          </div>
          {subtitle ? (
            <div style={{ color, opacity: 0.7, fontSize: 28 * k, marginTop: 4 * k }}>
              {subtitle}
            </div>
          ) : null}
        </div>
      </div>
    </AbsoluteFill>
  );
};

const CALLOUT_DIRECTION = {
  top: [0.35, -1],
  bottom: [0.35, 1],
  left: [-1, -0.35],
  right: [1, -0.35],
} as const;

export const Callout: React.FC<{
  text: string;
  x: number;
  y: number;
  side?: "top" | "bottom" | "left" | "right";
  color?: string;
  accent?: string;
  fontFamily?: string;
}> = ({ text, x, y, side = "top", color = TEXT_COLOR, accent = CALLOUT_COLOR, fontFamily }) => {
  const frame = useCurrentFrame();
  const { width, height, durationInFrames } = useVideoConfig();
  const { k } = useOverlayFrame();
  const draw = interpolate(frame, [0, 18], [0, 1], {
    ...CLAMPED,
    easing: Easing.out(Easing.cubic),
  });
  const off = 170 * k;
  const [dx, dy] = CALLOUT_DIRECTION[side];
  const sx = x * width + dx * off;
  const sy = y * height + dy * off;
  const len = Math.hypot(dx * off, dy * off);
  return (
    <AbsoluteFill style={{ opacity: exitOpacity(frame, durationInFrames) }}>
      <svg width={width} height={height} style={{ position: "absolute", inset: 0 }}>
        <path
          d={`M ${sx} ${sy} Q ${(sx + x * width) / 2 + 40 * k} ${(sy + y * height) / 2} ${x * width} ${y * height}`}
          fill="none"
          stroke={accent}
          strokeWidth={7 * k}
          strokeLinecap="round"
          strokeDasharray={len * 1.3}
          strokeDashoffset={len * 1.3 * (1 - draw)}
        />
        <circle cx={x * width} cy={y * height} r={14 * k * draw} fill={accent} />
      </svg>
      <div
        style={{
          position: "absolute",
          left: sx,
          top: sy,
          transform: `translate(-50%, ${dy > 0 ? "0" : "-100%"})`,
          padding: `${12 * k}px ${22 * k}px`,
          borderRadius: 14 * k,
          background: accent,
          color,
          fontFamily: fontOr(fontFamily, "main"),
          fontWeight: 700,
          fontSize: 34 * k,
          opacity: draw,
          whiteSpace: "nowrap",
        }}
      >
        {text}
      </div>
    </AbsoluteFill>
  );
};

type Caption = { text: string; startMs: number; endMs: number };

export function groupCaptions(captions: Caption[]): Caption[][] {
  const lines: Caption[][] = [];
  let line: Caption[] = [];
  for (const caption of captions) {
    line.push(caption);
    if (line.length >= MAX_WORDS_PER_LINE || /[.!?]$/.test(caption.text)) {
      lines.push(line);
      line = [];
    }
  }
  if (line.length) lines.push(line);
  return lines;
}

export const Captions: React.FC<{
  captions: Caption[];
  style?: "word-by-word" | "karaoke" | "line";
  position?: "bottom" | "center" | "top";
  fontSize?: number;
  color?: string;
  highlightColor?: string;
  fontFamily?: string;
}> = ({
  captions,
  style = "word-by-word",
  position = "bottom",
  fontSize = 56,
  color = CAPTION_COLOR,
  highlightColor = HIGHLIGHT_COLOR,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { k, insets } = useOverlayFrame();
  const tMs = framesToMs(frame, fps);
  const lines = useMemo(() => groupCaptions(captions), [captions]);
  const line = lines.find(
    (l) => tMs >= l[0].startMs && tMs <= l[l.length - 1].endMs + CAPTION_LINGER_MS,
  );
  if (!line) return null;
  return (
    <AbsoluteFill
      style={{
        justifyContent: JUSTIFY_AT[position],
        alignItems: "center",
        ...overlayPadding(90 * k, verticalInsets(insets)),
        paddingLeft: "10%",
        paddingRight: "10%",
      }}
    >
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          justifyContent: "center",
          gap: `0 ${fontSize * 0.28 * k}px`,
          fontFamily: fontOr(fontFamily, "main"),
          fontSize: fontSize * k,
          fontWeight: 800,
          textShadow: "0 4px 24px rgba(0,0,0,0.55)",
          padding: `${10 * k}px ${22 * k}px`,
          borderRadius: 16 * k,
          background: "rgba(0,0,0,0.35)",
        }}
      >
        {line.map((w, i) => {
          const active = tMs >= w.startMs && tMs < w.endMs;
          const spoken = tMs >= w.startMs;
          if (style === "word-by-word" && !spoken) return null;
          const pop = spring({
            frame: frame - msToFrames(w.startMs, fps, 0),
            fps,
            config: { damping: 14, stiffness: 220 },
          });
          const lit = { karaoke: spoken, "word-by-word": active, line: false }[style];
          return (
            <span
              key={i}
              style={{
                color: lit ? highlightColor : color,
                display: "inline-block",
                transform: style === "word-by-word" ? `scale(${0.85 + pop * 0.15})` : undefined,
                opacity: style === "karaoke" && !spoken ? 0.55 : 1,
              }}
            >
              {w.text}
            </span>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};
