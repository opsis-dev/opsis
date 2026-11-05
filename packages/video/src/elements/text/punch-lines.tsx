import { Easing } from "@opsis/runtime/easing.ts";
import { isRendering } from "@opsis/runtime/environment.ts";
import { holdRender } from "@opsis/runtime/holds.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { Fragment, useEffect, useMemo, useRef, useState } from "react";
import type React from "react";
import { type ElementProps } from "#video/common/spec.ts";
import { isTheme, themeColors } from "#video/common/theme.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import { Item, resolveFont, type WarmTheme, useWarmTheme } from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

const SANS = '"Inter"';

type PunchLineStyle = "slide" | "punch";

interface PunchLinesProps {
  script?: string;
  styles?: string;
  sizes?: string;
  holds?: string;
  accentBeat?: number;

  lead?: number;

  fontSize?: number;
  fontWeight?: number | string;
  lineHeight?: number;
  letterSpacing?: string;
  wordSpacing?: string;

  ground?: string;
  ink?: string;
  accentColor?: string;

  theme?: Partial<WarmTheme>;
  mode?: "light" | "dark";
  fontFamily?: string;

  slideFrom?: number;
  slideKick?: number;
  slideSettle?: number;
  lineStagger?: number;
  slideDrop?: number;
  motionBlur?: number;

  punchFrom?: number;
  punchFrames?: number;
  wordStagger?: number;
  pushFrom?: number;
  pushRate?: number;

  rushLead?: number;
  rushFrames?: number;
  rushTravel?: number;

  fit?: number;
  speed?: number;
  className?: string;
}

const PUNCH_EASE = Easing.bezier(0.65, 0, 0.35, 1);

const RUSH_EASE = Easing.bezier(1, 0, 0.48, 1);

function slideSwing(from: number, kick: number, settle: number): number {
  const peakAt = kick > 0 ? Math.max(0, 1 / settle - from / kick) : 0;
  const peak = (from + kick * peakAt) * Math.exp(-settle * peakAt);
  return Math.max(Math.abs(from), Math.abs(peak));
}

function list(s: string | undefined): string[] {
  return (s ?? "")
    .split(",")
    .map((v) => v.trim())
    .filter(Boolean);
}

function at<T>(values: T[], i: number, fallback: T): T {
  return values.length === 0 ? fallback : (values[Math.min(i, values.length - 1)] as T);
}

interface PunchBeat {
  lines: string[][];
  style: PunchLineStyle;
  size: number;
  accent: boolean;
  start: number;
  end: number;
  revealAt: number;
}

function planPunchLines(
  script: string,
  styles: string,
  sizes: string,
  holds: string,
  accentBeat: number,
  lead: number,
): PunchBeat[] {
  const texts = script
    .split("|")
    .map((s) => s.trim())
    .filter(Boolean);
  const styleList = list(styles) as PunchLineStyle[];
  const sizeList = list(sizes).map(Number).filter(Number.isFinite);
  const holdList = list(holds).map(Number).filter(Number.isFinite);

  const beats: PunchBeat[] = [];
  let revealAt = 0;
  for (let i = 0; i < texts.length; i++) {
    const style = at(styleList, i, "punch");
    const hold = Math.max(1, at(holdList, i, 48));
    beats.push({
      lines: (texts[i] as string)
        .split("/")
        .map((l) => l.trim())
        .filter(Boolean)
        .map((l) => l.split(/\s+/)),
      style: style === "slide" ? "slide" : "punch",
      size: at(sizeList, i, 1),
      accent: i + 1 === accentBeat,
      start: i === 0 ? 0 : revealAt - lead,
      revealAt,
      end: revealAt + hold,
    });
    revealAt += hold;
  }
  return beats;
}

export function punchLinesDuration(script: string, holds: string): number {
  const n = script
    .split("|")
    .map((s) => s.trim())
    .filter(Boolean).length;
  const holdList = list(holds).map(Number).filter(Number.isFinite);
  let total = 0;
  for (let i = 0; i < n; i++) total += Math.max(1, at(holdList, i, 48));
  return total;
}

interface FaceMetrics {
  baseline: number;
  inkAscent: number[];
  inkDescent: number[];
}

function PunchLinesView({
  script = "Record it. / Edit it. | One take. | Ship.",
  styles = "slide,punch",
  sizes = "1,0.945,2.7",
  holds = "48",
  accentBeat = 3,
  lead = 4,

  fontSize = 93,
  fontWeight = 600,
  lineHeight = 0.89,
  letterSpacing = "-0.022em",
  wordSpacing = "-0.02em",

  ground,
  ink,
  accentColor = "#ff1c8e",

  theme,
  mode,
  fontFamily,

  slideFrom = -0.737,
  slideKick = 7.34,
  slideSettle = 2.846,
  lineStagger = 3.5,
  slideDrop = 0.313,
  motionBlur = 1,

  punchFrom = 0.32,
  punchFrames = 15,
  wordStagger = 2.8,
  pushFrom = 0.778,
  pushRate = 0.205,

  rushLead = 12.2,
  rushFrames = 18,
  rushTravel = 0.85,

  fit = 0.92,
  speed = 1,
  className,
}: PunchLinesProps) {
  const frame = useAuthoredFrame() * speed;
  const { width, fps } = useAuthoredConfig();
  const t = useWarmTheme(theme, mode);
  const face = resolveFont(fontFamily ?? t.fontFamily) ?? SANS;
  const paper = ground ?? t.background;
  const type = ink ?? t.foreground;

  const beats = useMemo(
    () => planPunchLines(script, styles, sizes, holds, accentBeat, lead),
    [script, styles, sizes, holds, accentBeat, lead],
  );

  const probeRef = useRef<HTMLSpanElement>(null);
  const baselineRef = useRef<HTMLSpanElement>(null);
  const lineRefs = useRef<(HTMLSpanElement | null)[]>([]);
  const [release] = useState(() => holdRender("punch-lines: measure the face"));
  const [metrics, setMetrics] = useState<FaceMetrics | null>(null);
  const [lineWidths, setLineWidths] = useState<number[] | null>(null);

  useEffect(() => {
    const probe = probeRef.current;
    if (!probe) {
      release();
      return;
    }
    const baseline = baselineRef.current?.offsetTop ?? 88;
    const firstLines = beats.map((b) => (b.lines[0] ?? []).join(" "));
    const lastLines = beats.map((b) => (b.lines[b.lines.length - 1] ?? []).join(" "));
    let inkAscent = firstLines.map(() => 70);
    let inkDescent = lastLines.map(() => 0);
    try {
      const ctx = document.createElement("canvas").getContext("2d");
      if (ctx) {
        ctx.font = `${fontWeight} 100px ${face}`;
        inkAscent = firstLines.map((text) => {
          const a = ctx.measureText(text).actualBoundingBoxAscent;
          return a > 0 ? a : 70;
        });
        inkDescent = lastLines.map((text) =>
          Math.max(0, ctx.measureText(text).actualBoundingBoxDescent),
        );
      }
    } catch {}
    const lineCount = beats.reduce((n, b) => n + b.lines.length, 0);
    setLineWidths(lineRefs.current.slice(0, lineCount).map((el) => (el ? el.offsetWidth : 0)));
    setMetrics({ baseline, inkAscent, inkDescent });
    // oxlint-disable-next-line react/exhaustive-effect-dependencies
  }, [release, face, fontWeight, beats, fontSize, letterSpacing, wordSpacing]);

  useEffect(() => {
    if (metrics) release();
  }, [metrics, release]);

  const M: FaceMetrics = metrics ?? {
    baseline: 88,
    inkAscent: beats.map(() => 70),
    inkDescent: beats.map(() => 0),
  };

  const lineIndex: number[] = [];
  {
    let n = 0;
    for (const beat of beats) {
      lineIndex.push(n);
      n += beat.lines.length;
    }
  }
  const allLines = ([] as { words: string[]; size: number }[]).concat(
    ...beats.map((beat) => beat.lines.map((words) => ({ words, size: beat.size }))),
  );

  return (
    <AbsoluteFill
      className={className}
      style={{
        backgroundColor: paper,
        fontFamily: face,
        fontWeight,
        textRendering: "geometricPrecision",
        WebkitFontSmoothing: "antialiased",
      }}
    >
      <div
        aria-hidden
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          visibility: "hidden",
          pointerEvents: "none",
          whiteSpace: "nowrap",
        }}
      >
        <span ref={probeRef} style={{ fontSize: 100, lineHeight: `${lineHeight * 100}px` }}>
          Hxg
          <span ref={baselineRef} style={{ display: "inline-block", width: 0, height: 0 }} />
        </span>
      </div>
      <div
        aria-hidden
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          visibility: "hidden",
          pointerEvents: "none",
          whiteSpace: "nowrap",
        }}
      >
        {allLines.map((line, i) => (
          <span
            key={i}
            ref={(el) => {
              lineRefs.current[i] = el;
            }}
            style={{
              display: "inline-block",
              fontSize: fontSize * line.size,
              lineHeight: `${lineHeight}em`,
              letterSpacing,
              wordSpacing,
            }}
          >
            {line.words.join(" ")}
          </span>
        ))}
      </div>

      {beats.map((beat, i) => (
        <Card
          key={i}
          beat={beat}
          frame={frame}
          fps={fps}
          width={width}
          fontSize={fontSize}
          lineHeight={lineHeight}
          letterSpacing={letterSpacing}
          wordSpacing={wordSpacing}
          metrics={M}
          beatIndex={i}
          widths={
            lineWidths
              ? lineWidths.slice(
                  lineIndex[i] as number,
                  (lineIndex[i] as number) + beat.lines.length,
                )
              : null
          }
          color={beat.accent ? accentColor : type}
          fit={fit}
          slideFrom={slideFrom}
          slideKick={slideKick}
          slideSettle={slideSettle}
          lineStagger={lineStagger}
          slideDrop={slideDrop}
          motionBlur={motionBlur}
          punchFrom={punchFrom}
          punchFrames={punchFrames}
          wordStagger={wordStagger}
          pushFrom={pushFrom}
          pushRate={pushRate}
          rushLead={rushLead}
          rushFrames={rushFrames}
          rushTravel={rushTravel}
        />
      ))}
    </AbsoluteFill>
  );
}

function Card({
  beat,
  frame,
  fps,
  width,
  fontSize,
  lineHeight,
  letterSpacing,
  wordSpacing,
  metrics,
  beatIndex,
  widths,
  color,
  fit,
  slideFrom,
  slideKick,
  slideSettle,
  lineStagger,
  slideDrop,
  motionBlur,
  punchFrom,
  punchFrames,
  wordStagger,
  pushFrom,
  pushRate,
  rushLead,
  rushFrames,
  rushTravel,
}: {
  beat: PunchBeat;
  frame: number;
  fps: number;
  width: number;
  fontSize: number;
  lineHeight: number;
  letterSpacing: string;
  wordSpacing: string;
  metrics: FaceMetrics;
  beatIndex: number;
  widths: number[] | null;
  color: string;
  fit: number;
  slideFrom: number;
  slideKick: number;
  slideSettle: number;
  lineStagger: number;
  slideDrop: number;
  motionBlur: number;
  punchFrom: number;
  punchFrames: number;
  wordStagger: number;
  pushFrom: number;
  pushRate: number;
  rushLead: number;
  rushFrames: number;
  rushTravel: number;
}) {
  if (frame < beat.revealAt || frame >= beat.end) return null;

  const t = (frame - beat.start) / fps;

  const natural = widths?.length ? Math.max(...widths) : 0;
  const maxPush =
    beat.style === "punch" ? pushFrom + pushRate * ((beat.end - beat.start) / fps) : 1;
  const swing =
    beat.style === "slide"
      ? slideSwing(slideFrom, slideKick, slideSettle) * fontSize * beat.size
      : 0;
  const need = natural * maxPush + 2 * swing;
  const shrink = need > width * fit && need > 0 ? (width * fit) / need : 1;
  const em = fontSize * beat.size * shrink;
  const L = lineHeight * em;

  const baseline = (metrics.baseline / 100) * em;
  const rise = ((metrics.inkAscent[beatIndex] ?? 70) / 100) * em;
  const fall = ((metrics.inkDescent[beatIndex] ?? 0) / 100) * em;
  const dy = L / 2 - baseline + (rise - fall) / 2 + (beat.style === "slide" ? slideDrop * em : 0);

  const push = beat.style === "punch" ? pushFrom + pushRate * Math.max(0, t) : 1;
  const rushT = (frame - (beat.end - rushLead)) / fps;
  const rush =
    beat.style === "slide" && rushT > 0
      ? 1 / (1 - rushTravel * RUSH_EASE(Math.min(1, rushT / (rushFrames / fps))))
      : 1;

  const lineStarts = beat.lines.map((_, li) =>
    beat.lines.slice(0, li).reduce((count, words) => count + words.length, 0),
  );

  return (
    <AbsoluteFill
      style={{
        alignItems: "center",
        justifyContent: "center",
        color,
        transform: `scale(${push * rush})`,
        transformOrigin: "50% 50%",
        ...(isRendering() ? null : { willChange: "transform" as const }),
      }}
    >
      <Item index={beatIndex}>
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            transform: `translateY(${dy}px)`,
          }}
        >
          {beat.lines.map((words, li) => {
            const lt = t - (li * lineStagger) / fps;
            function slide(u: number) {
              return beat.style !== "slide"
                ? 0
                : u <= 0
                  ? slideFrom * em
                  : (slideFrom * em + slideKick * em * u) * Math.exp(-slideSettle * u);
            }
            const x = slide(lt);
            const sigma = (motionBlur * Math.abs(x - slide(lt - 1 / fps))) / Math.sqrt(12);
            const blurId = `punch-lines-${beatIndex}-${li}`;

            return (
              <div
                key={li}
                style={{
                  whiteSpace: "nowrap",
                  fontSize: em,
                  lineHeight: `${L}px`,
                  letterSpacing,
                  wordSpacing,
                  transform: `translateX(${x}px)`,
                  filter: sigma > 0.2 ? `url(#${blurId})` : undefined,
                }}
              >
                {sigma > 0.2 ? (
                  <svg aria-hidden width={0} height={0} style={{ position: "absolute" }}>
                    <title>motion blur</title>
                    <filter
                      id={blurId}
                      x="-25%"
                      y="-25%"
                      width="150%"
                      height="150%"
                      colorInterpolationFilters="sRGB"
                    >
                      <feGaussianBlur stdDeviation={`${sigma} 0`} />
                    </filter>
                  </svg>
                ) : null}
                {words.map((word, wi) => {
                  const w = (lineStarts[li] ?? 0) + wi;
                  const wt = t - (w * wordStagger) / fps;
                  const p =
                    beat.style === "punch"
                      ? PUNCH_EASE(Math.max(0, Math.min(1, wt / (punchFrames / fps))))
                      : 1;
                  const s = beat.style === "punch" ? punchFrom + (1 - punchFrom) * p : 1;
                  return (
                    <Fragment key={w}>
                      {wi > 0 ? " " : null}
                      <span
                        style={{
                          display: "inline-block",
                          transform: `scale(${s})`,
                          transformOrigin: `50% ${L / 2}px`,
                        }}
                      >
                        {word}
                      </span>
                    </Fragment>
                  );
                })}
              </div>
            );
          })}
        </div>
      </Item>
    </AbsoluteFill>
  );
}

export function PunchLines(props: ElementProps & { children?: React.ReactNode }) {
  const { theme } = props;
  if (!isTheme(theme))
    return (
      <PunchLinesView {...(props as unknown as React.ComponentProps<typeof PunchLinesView>)} />
    );
  return (
    <WarmUIProvider mode={theme.mode}>
      <PunchLinesView
        {...(props as unknown as React.ComponentProps<typeof PunchLinesView>)}
        theme={themeColors(theme)}
        mode={theme.mode}
      />
    </WarmUIProvider>
  );
}
