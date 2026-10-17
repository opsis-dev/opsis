import React, { useMemo } from "react";
import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { clamp, entranceStyle, exitOpacity, type Entrance } from "./anim";
import { isLandscape, useOverlayFrame } from "./frame";
import { roleStack } from "../../shared/fonts";
import { msToFrames } from "../../shared/timeline/time";

const HEADING_FONT = roleStack("heading");
const MAIN_FONT = roleStack("main");

const Bg: React.FC<{ background?: string }> = ({ background }) =>
  background && background !== "transparent" ? <AbsoluteFill style={{ background }} /> : null;

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
  color = "#fafafa",
  accent = "#a78bfa",
  background,
  fontSize = 112,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const out = exitOpacity(frame, durationInFrames);
  return (
    <AbsoluteFill>
      <Bg background={background} />
      <AbsoluteFill
        style={{
          alignItems: align === "center" ? "center" : "flex-start",
          justifyContent: "center",
          padding: "0 9%",
          textAlign: align,
          fontFamily: fontFamily || HEADING_FONT,
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
  color = "#fafafa",
  emphasisColor = "#facc15",
  background,
  fontSize = 104,
  wordStagger = 4,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const words = useMemo(
    () =>
      text
        .split(/\s+/)
        .filter(Boolean)
        .map((w) => ({
          word: w.replace(/^\*|\*$/g, ""),
          em: /^\*.*\*$/.test(w) || w.startsWith("*") || w.endsWith("*"),
        })),
    [text],
  );
  return (
    <AbsoluteFill>
      <Bg background={background} />
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
            fontFamily: fontFamily || HEADING_FONT,
            fontSize,
            fontWeight: 780,
            letterSpacing: "-0.04em",
            lineHeight: 1.05,
          }}
        >
          {words.map(({ word, em }, i) => {
            const f = frame - i * wordStagger;
            const s = spring({ frame: f, fps, config: { damping: 18, stiffness: 160 } });
            const sweep = interpolate(f, [8, 22], [0, 1], {
              ...clamp,
              easing: Easing.out(Easing.cubic),
            });
            return (
              <span
                key={i}
                style={{
                  position: "relative",
                  display: "inline-block",
                  color: em ? emphasisColor : color,
                  opacity: interpolate(f, [0, 6], [0, 1], clamp),
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
          })}
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
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
  accent = "#a78bfa",
  color = "#fafafa",
  panel = "rgba(10,10,12,0.72)",
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames, width, height } = useVideoConfig();
  const { k, insets } = useOverlayFrame();
  const enter = spring({ frame, fps, config: { damping: 200 } });
  const exit = spring({ frame: frame - (durationInFrames - 14), fps, config: { damping: 200 } });
  const top = position.startsWith("top") || !isLandscape(width, height);
  const right = position.endsWith("right");
  const dir = right ? 1 : -1;
  const x = (1 - enter) * 120 * dir + exit * 120 * dir;
  return (
    <AbsoluteFill
      style={{
        justifyContent: top ? "flex-start" : "flex-end",
        alignItems: right ? "flex-end" : "flex-start",
        padding: 80 * k,
        paddingTop: 80 * k + insets.top,
        paddingBottom: 80 * k + insets.bottom,
        paddingLeft: 80 * k + insets.left,
        paddingRight: 80 * k + insets.right,
      }}
    >
      <div
        style={{
          display: "flex",
          gap: 18 * k,
          alignItems: "stretch",
          transform: `translateX(${x * k}px)`,
          opacity: Math.min(enter, 1 - exit),
          fontFamily: fontFamily || MAIN_FONT,
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

export const Callout: React.FC<{
  text: string;
  x: number;
  y: number;
  side?: "top" | "bottom" | "left" | "right";
  color?: string;
  accent?: string;
  fontFamily?: string;
}> = ({ text, x, y, side = "top", color = "#fafafa", accent = "#f43f5e", fontFamily }) => {
  const frame = useCurrentFrame();
  const { width, height, durationInFrames } = useVideoConfig();
  const k = width / 1920;
  const draw = interpolate(frame, [0, 18], [0, 1], { ...clamp, easing: Easing.out(Easing.cubic) });
  const off = 170 * k;
  const [dx, dy] =
    side === "top"
      ? [0.35, -1]
      : side === "bottom"
        ? [0.35, 1]
        : side === "left"
          ? [-1, -0.35]
          : [1, -0.35];
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
          fontFamily: fontFamily || MAIN_FONT,
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

export const Captions: React.FC<{
  captions: Caption[];
  style?: "word-by-word" | "karaoke" | "line";
  position?: "bottom" | "center" | "top";
  fontSize?: number;
  color?: string;
  highlightColor?: string;
  fontFamily?: string;
}> = ({
  captions = [],
  style = "word-by-word",
  position = "bottom",
  fontSize = 56,
  color = "#ffffff",
  highlightColor = "#facc15",
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { k, insets } = useOverlayFrame();
  const tMs = (frame / fps) * 1000;
  const lines = useMemo(() => {
    const out: Caption[][] = [];
    let cur: Caption[] = [];
    for (const c of captions) {
      cur.push(c);
      if (cur.length >= 6 || /[.!?]$/.test(c.text)) {
        out.push(cur);
        cur = [];
      }
    }
    if (cur.length) out.push(cur);
    return out;
  }, [captions]);
  const line = lines.find((l) => tMs >= l[0].startMs && tMs <= l[l.length - 1].endMs + 250);
  if (!line) return null;
  return (
    <AbsoluteFill
      style={{
        justifyContent:
          position === "bottom" ? "flex-end" : position === "top" ? "flex-start" : "center",
        alignItems: "center",
        padding: `${90 * k}px 10%`,
        paddingTop: 90 * k + insets.top,
        paddingBottom: 90 * k + insets.bottom,
      }}
    >
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          justifyContent: "center",
          gap: `0 ${fontSize * 0.28 * k}px`,
          fontFamily: fontFamily || MAIN_FONT,
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
          const c =
            style === "line"
              ? color
              : style === "karaoke"
                ? spoken
                  ? highlightColor
                  : color
                : active
                  ? highlightColor
                  : color;
          return (
            <span
              key={i}
              style={{
                color: c,
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
