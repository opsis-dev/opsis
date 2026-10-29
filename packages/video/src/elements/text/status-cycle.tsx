import { useEffect, useMemo, useRef, useState } from "react";
import {
  AbsoluteFill,
  continueRender,
  delayRender,
  Easing,
  getRemotionEnvironment,
  interpolate,
  spring,
} from "remotion";
import { isTheme, themeColors } from "#video/common/theme.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import {
  Item,
  mixOklch,
  resolveFont,
  type WarmTheme,
  useWarmTheme,
} from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

const SANS = '"Inter"';

const CLAMP = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
} as const;

const PILL_R = 0.229;
const CHIP_PAD_Y = 0.165;
const CHIP_R = 0.162;
const CHIP_BORDER = 0.017;
const ACT1_FIT = 0.92;

const WIDTH_EASE = Easing.bezier(0.65, 0.2, 0.3, 1.5);
const WIDTH_SECONDS = 0.6;

const ROLL_EASE = Easing.bezier(0.45, 0, 0.55, 1);
const ROLL_SECONDS = 0.5;
const ROLL_TRAVEL = 1.02;

const FADE_SECONDS = 0.3;
const EXIT_SECONDS = 0.2;
const SCROLL_TAU = 0.06;

const LETTER_SPRING = { damping: 5, stiffness: 100, mass: 1 } as const;

export function toList(value?: string[] | string): string[] {
  const list = typeof value === "string" ? value.split(",") : (value ?? []);
  return list.map((s) => s.trim()).filter(Boolean);
}

export function settle(frames: number, tau: number): number {
  if (frames <= 0) return 0;
  return 1 - Math.exp(-frames / tau);
}

export function statusAt({
  frame,
  introFrames,
  statusHold,
  count,
}: {
  frame: number;
  introFrames: number;
  statusHold: number;
  count: number;
}): { index: number; since: number } {
  if (count <= 0) return { index: 0, since: 0 };
  let index = 0;
  for (let i = 1; i < count; i += 1) {
    if (frame >= introFrames + (i - 1) * statusHold) index = i;
  }
  return { index, since: frame - (introFrames + (index - 1) * statusHold) };
}

export function chipAt({
  frame,
  beat,
  stagger,
  count,
}: {
  frame: number;
  beat: number;
  stagger: number;
  count: number;
}): { arrived: number; since: number } {
  if (count <= 0 || frame < beat) return { arrived: 0, since: 0 };
  const arrived = Math.min(count, Math.floor((frame - beat) / stagger) + 1);
  return { arrived, since: frame - beat - (arrived - 1) * stagger };
}

export function fitScale(measured: number, budget: number): number {
  if (!(measured > 0) || !(budget > 0)) return 1;
  return Math.min(1, budget / measured);
}

export interface StatusCycleMotion {
  widthEase: (t: number) => number;
  widthSeconds: number;
  rollEase: (t: number) => number;
  rollSeconds: number;
  rollTravel: number;
  fadeSeconds: number;
  exitSeconds: number;
  scrollTau: number;
  letterSpring: { damping: number; stiffness: number; mass: number };
  letterStagger: number;
}

export const STATUS_CYCLE_MOTION: StatusCycleMotion = {
  widthEase: WIDTH_EASE,
  widthSeconds: WIDTH_SECONDS,
  rollEase: ROLL_EASE,
  rollSeconds: ROLL_SECONDS,
  rollTravel: ROLL_TRAVEL,
  fadeSeconds: FADE_SECONDS,
  exitSeconds: EXIT_SECONDS,
  scrollTau: SCROLL_TAU,
  letterSpring: LETTER_SPRING,
  letterStagger: 1,
};

interface Metrics {
  letters: { left: number; width: number }[];
  prefixWidth: number;
  baseline: number;
  statusWidths: number[];
  chipWidths: number[];
}

function StatusCycleView({
  prefix,
  statuses,
  chips,
  fieldColor,
  pageColor,
  pillColor,
  pillLabelColor,
  prefixColor,
  chipFills,
  chipTextColor,
  chipBorderColor,
  fontWeight,
  chipFontWeight,
  pillRadius,
  chipRadius,
  chipBorderWidth,
  pillCenterY,
  fontSize,
  chipFontSize,
  fontFamily,
  statusHold,
  introFrames,
  chipStagger,
  startAt,
  speed,
  pillHeight,
  pillPadding,
  pillGap,
  chipPadding,
  chipInset,
  chipGap,
  pillResizeTime,
  rollTime,
  fieldFadeTime,
  letterStagger,
  introZoom,
  colors,
  mode,
}: Omit<ElementSettings["status-cycle"], "theme"> & {
  colors?: Partial<WarmTheme>;
  mode?: "light" | "dark";
}) {
  const frame = useAuthoredFrame() * speed - startAt;
  const { fps, width, height } = useAuthoredConfig();
  const t = useWarmTheme(colors, mode);
  const face = resolveFont(fontFamily || t.fontFamily) ?? SANS;

  const labels = useMemo(() => toList(statuses), [statuses]);
  const chipLabels = useMemo(() => toList(chips), [chips]);
  const glyphs = useMemo(() => [...prefix], [prefix]);

  const field = fieldColor ?? t.primary;
  const page = pageColor ?? t.background;
  const pillFill = pillColor ?? t.foreground;
  const prefixInk = prefixColor ?? t.foreground;
  const labelInk = pillLabelColor ?? mixOklch(field, t.card, 0.62);
  const chipInk = chipTextColor ?? t.foreground;
  const chipEdge = chipBorderColor || mixOklch(t.border, t.foreground, 0.28);
  const m = {
    ...STATUS_CYCLE_MOTION,
    widthSeconds: pillResizeTime,
    rollSeconds: rollTime,
    fadeSeconds: fieldFadeTime,
    letterStagger,
  };
  const fills = toList(chipFills);
  const fillCycle = fills.length > 0 ? fills : [t.card, mixOklch(t.card, field, 0.14), field];

  const nominalEm = fontSize;
  const nominalChipEm = chipFontSize;

  const measureKey = JSON.stringify([
    glyphs,
    labels,
    chipLabels,
    nominalEm,
    nominalChipEm,
    face,
    fontWeight,
    chipFontWeight,
  ]);
  const [measured, setMeasured] = useState<{ key: string; value: Metrics } | null>(null);
  const metrics = measured?.key === measureKey ? measured.value : null;
  const [handle] = useState(() => delayRender("status-cycle: measuring"));
  const released = useRef(false);
  const letterRefs = useRef<(HTMLSpanElement | null)[]>([]);
  const prefixRef = useRef<HTMLSpanElement | null>(null);
  const baselineRef = useRef<HTMLSpanElement | null>(null);
  const statusRefs = useRef<(HTMLSpanElement | null)[]>([]);
  const chipRefs = useRef<(HTMLSpanElement | null)[]>([]);

  useEffect(() => {
    if (metrics) {
      if (!released.current) continueRender(handle);
      released.current = true;
      return;
    }
    const prefixWidth = prefixRef.current?.offsetWidth ?? 0;
    if (prefixWidth <= 0) return;
    setMeasured({
      key: measureKey,
      value: {
        letters: glyphs.map((_, i) => ({
          left: letterRefs.current[i]?.offsetLeft ?? 0,
          width: letterRefs.current[i]?.offsetWidth ?? 0,
        })),
        prefixWidth,
        baseline: baselineRef.current?.offsetTop ?? 0,
        statusWidths: labels.map((_, i) => statusRefs.current[i]?.offsetWidth ?? 0),
        chipWidths: chipLabels.map((_, i) => chipRefs.current[i]?.offsetWidth ?? 0),
      },
    });
  }, [metrics, handle, glyphs, labels, chipLabels, measureKey]);

  const swapAt = (i: number) => introFrames + (i - 1) * statusHold;
  const act1End = swapAt(labels.length) + statusHold;
  const fadeStart = act1End;
  const fadeFrames = m.fadeSeconds * fps;
  const beat = fadeStart + fadeFrames / 2;

  const fadeP = interpolate(frame, [fadeStart, fadeStart + fadeFrames], [0, 1], CLAMP);

  const rawPad = pillPadding * pillHeight * nominalEm;
  const rawGap = pillGap * pillHeight * nominalEm;
  const rawLockup =
    (metrics?.prefixWidth ?? 0) +
    rawGap +
    Math.max(0, ...(metrics?.statusWidths ?? [0])) +
    2 * rawPad;
  const act1Fit = metrics ? fitScale(rawLockup, ACT1_FIT * width) : 1;

  const rawChip = Math.max(0, ...(metrics?.chipWidths ?? [0])) + 2 * chipPadding * nominalChipEm;
  const chipFit = metrics ? fitScale(rawChip, (1 - 2 * chipInset) * width) : 1;

  const em = nominalEm * act1Fit;
  const chipEm = nominalChipEm * chipFit;
  const pillH = pillHeight * em;
  const pillPad = pillPadding * pillH;
  const gap = pillGap * pillH;
  const prefixWidth = (metrics?.prefixWidth ?? 0) * act1Fit;

  const pillWidths = (metrics?.statusWidths ?? labels.map(() => 0)).map(
    (w) => w * act1Fit + 2 * pillPad,
  );

  const widthSeconds = m.widthSeconds * fps;
  const rollSeconds = m.rollSeconds * fps;
  const { index, since } = statusAt({
    frame,
    introFrames,
    statusHold,
    count: labels.length,
  });
  const widthP =
    index === 0
      ? m.widthEase(Math.min(1, Math.max(0, frame / introFrames)))
      : m.widthEase(Math.min(1, Math.max(0, since / widthSeconds)));
  const rollP = index === 0 ? 1 : m.rollEase(Math.min(1, Math.max(0, since / rollSeconds)));

  const pillW = interpolate(
    widthP,
    [0, 1],
    [index === 0 ? 0 : (pillWidths[index - 1] ?? 0), pillWidths[index] ?? 0],
  );

  const groupScale = spring({
    frame,
    fps,
    config: { damping: 26, stiffness: 90, mass: 1.4 },
    from: introZoom,
    to: 1,
  });

  const exitP = settle((frame - beat) * (1 / m.exitSeconds / fps), 0.28);
  const exitY = frame < beat ? 0 : -exitP * 0.62 * height;

  const total = prefixWidth + gap + pillW;
  const originX = (width - total) / 2;
  const cy = pillCenterY * height;

  const promote = getRemotionEnvironment().isRendering
    ? null
    : ({ willChange: "transform" } as const);

  const typeStyle = {
    fontFamily: face,
    fontWeight,
    textRendering: "geometricPrecision" as const,
    whiteSpace: "nowrap" as const,
  };

  return (
    <AbsoluteFill style={{ backgroundColor: page, overflow: "hidden" }}>
      <AbsoluteFill style={{ backgroundColor: field, opacity: 1 - fadeP }} />
      {metrics ? null : (
        <Measuring
          glyphs={glyphs}
          labels={labels}
          chipLabels={chipLabels}
          em={em}
          chipEm={chipEm}
          typeStyle={typeStyle}
          letterRefs={letterRefs}
          prefixRef={prefixRef}
          baselineRef={baselineRef}
          statusRefs={statusRefs}
          chipRefs={chipRefs}
        />
      )}

      {metrics && frame < beat + m.exitSeconds * fps ? (
        <div
          style={{
            position: "absolute",
            left: 0,
            top: 0,
            transform: `translateY(${exitY}px) scale(${groupScale})`,
            transformOrigin: `${width / 2}px ${cy}px`,
            width,
            height,
            ...promote,
          }}
        >
          <Prefix
            glyphs={glyphs}
            metrics={metrics}
            frame={frame}
            fps={fps}
            em={em}
            fit={act1Fit}
            spring={m.letterSpring}
            stagger={m.letterStagger}
            x={originX}
            cy={cy}
            color={prefixInk}
            typeStyle={typeStyle}
          />
          <Pill
            labels={labels}
            index={index}
            rollP={rollP}
            x={originX + prefixWidth + gap}
            cy={cy}
            width={pillW}
            height={pillH}
            em={em}
            fill={pillFill}
            ink={labelInk}
            radius={pillRadius ?? PILL_R * pillH}
            travel={m.rollTravel * pillH}
            typeStyle={typeStyle}
          />
        </div>
      ) : null}

      {metrics ? (
        <Chips
          labels={chipLabels}
          widths={metrics.chipWidths.map((w) => w * chipFit)}
          frame={frame}
          fps={fps}
          beat={beat}
          stagger={chipStagger}
          em={chipEm}
          width={width}
          height={height}
          fills={fillCycle}
          ink={chipInk}
          edge={chipEdge}
          radius={chipRadius}
          borderWidth={chipBorderWidth}
          weight={chipFontWeight}
          tau={m.scrollTau}
          typeStyle={typeStyle}
          padding={chipPadding}
          inset={chipInset}
          spacing={chipGap}
        />
      ) : null}
    </AbsoluteFill>
  );
}

function Prefix({
  glyphs,
  metrics,
  frame,
  fps,
  em,
  fit,
  spring: springConfig,
  stagger,
  x,
  cy,
  color,
  typeStyle,
}: {
  glyphs: string[];
  metrics: Metrics;
  frame: number;
  fps: number;
  em: number;
  fit: number;
  spring: { damping: number; stiffness: number; mass: number };
  stagger: number;
  x: number;
  cy: number;
  color: string;
  typeStyle: React.CSSProperties;
}) {
  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: cy - em,
        height: 2 * em,
        color,
        fontSize: em,
        lineHeight: 2,
        ...typeStyle,
      }}
    >
      {glyphs.map((g, i) => {
        const p = spring({
          frame: frame - i * stagger,
          fps,
          config: springConfig,
        });
        const s = interpolate(p, [0, 1], [0.1, 1]);
        return (
          <span
            key={i}
            style={{
              position: "absolute",
              left: (metrics.letters[i]?.left ?? 0) * fit,
              top: 0,
              transform: `scale(${s})`,
              transformOrigin: `50% ${metrics.baseline * fit}px`,
              ...typeStyle,
            }}
          >
            {g === " " ? " " : g}
          </span>
        );
      })}
    </div>
  );
}

function Pill({
  labels,
  index,
  rollP,
  x,
  cy,
  width,
  height,
  em,
  fill,
  ink,
  radius,
  travel,
  typeStyle,
}: {
  labels: string[];
  index: number;
  rollP: number;
  x: number;
  cy: number;
  width: number;
  height: number;
  em: number;
  fill: string;
  ink: string;
  radius: number;
  travel: number;
  typeStyle: React.CSSProperties;
}) {
  const showOutgoing = rollP < 1 && index > 0;

  const label = (text: string, offset: number) => (
    <div
      style={{
        position: "absolute",
        left: "50%",
        top: 0,
        height,
        display: "flex",
        alignItems: "center",
        transform: `translate(-50%, ${offset}px)`,
        color: ink,
        fontSize: em,
        ...typeStyle,
      }}
    >
      {text}
    </div>
  );

  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: cy - height / 2,
        width,
        height,
        borderRadius: radius,
        backgroundColor: fill,
        overflow: "hidden",
      }}
    >
      {showOutgoing ? label(labels[index - 1] ?? "", -rollP * travel) : null}
      {label(labels[index] ?? "", (1 - rollP) * travel)}
    </div>
  );
}

function Chips({
  labels,
  widths,
  frame,
  fps,
  beat,
  stagger,
  em,
  width,
  height,
  fills,
  ink,
  edge,
  radius,
  borderWidth,
  weight,
  tau: scrollTau,
  typeStyle,
  padding,
  inset,
  spacing,
}: {
  padding: number;
  inset: number;
  spacing: number;
  labels: string[];
  widths: number[];
  frame: number;
  fps: number;
  beat: number;
  stagger: number;
  em: number;
  width: number;
  height: number;
  fills: string[];
  ink: string;
  edge: string;
  radius?: number;
  borderWidth?: number;
  weight: string;
  tau: number;
  typeStyle: React.CSSProperties;
}) {
  if (frame < beat) return null;

  const chipH = em * (1 + 2 * CHIP_PAD_Y);
  const step = chipH + spacing * height;
  const tau = scrollTau * fps;

  const { arrived, since } = chipAt({
    frame,
    beat,
    stagger,
    count: labels.length,
  });
  const p = settle(since, tau);
  const fieldY = height - spacing * height - chipH - (arrived - 2 + p) * step;

  return (
    <div style={{ position: "absolute", inset: 0, overflow: "hidden" }}>
      {labels.slice(0, arrived).map((text, i) => {
        const w = (widths[i] ?? 0) + 2 * padding * em;
        const left = i % 2 === 0;
        const fill = fills[i % fills.length] ?? fills[0];
        return (
          <Item key={i} index={i}>
            <div
              style={{
                position: "absolute",
                top: 0,
                left: left ? inset * width : undefined,
                right: left ? undefined : inset * width,
                transform: `translateY(${fieldY + i * step}px)`,
                width: w,
                height: chipH,
                borderRadius: radius ?? CHIP_R * chipH,
                backgroundColor: fill,
                border: `${borderWidth ?? CHIP_BORDER * chipH}px solid ${edge}`,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: ink,
                fontSize: em,
                ...typeStyle,
                fontWeight: weight,
              }}
            >
              {text}
            </div>
          </Item>
        );
      })}
    </div>
  );
}

function Measuring({
  glyphs,
  labels,
  chipLabels,
  em,
  chipEm,
  typeStyle,
  letterRefs,
  prefixRef,
  baselineRef,
  statusRefs,
  chipRefs,
}: {
  glyphs: string[];
  labels: string[];
  chipLabels: string[];
  em: number;
  chipEm: number;
  typeStyle: React.CSSProperties;
  letterRefs: { current: (HTMLSpanElement | null)[] };
  prefixRef: { current: HTMLSpanElement | null };
  baselineRef: { current: HTMLSpanElement | null };
  statusRefs: { current: (HTMLSpanElement | null)[] };
  chipRefs: { current: (HTMLSpanElement | null)[] };
}) {
  return (
    <div
      aria-hidden
      style={{
        position: "absolute",
        visibility: "hidden",
        left: 0,
        top: 0,
        pointerEvents: "none",
        display: "flex",
        flexDirection: "column",
        alignItems: "flex-start",
      }}
    >
      <span ref={prefixRef} style={{ fontSize: em, lineHeight: 2, ...typeStyle }}>
        {glyphs.map((g, i) => (
          <span
            key={i}
            ref={(el) => {
              letterRefs.current[i] = el;
            }}
            style={{ display: "inline-block" }}
          >
            {g === " " ? " " : g}
          </span>
        ))}
        <span ref={baselineRef} style={{ display: "inline-block", width: 0, height: 0 }} />
      </span>
      {labels.map((text, i) => (
        <span
          key={i}
          ref={(el) => {
            statusRefs.current[i] = el;
          }}
          style={{ display: "inline-block", fontSize: em, ...typeStyle }}
        >
          {text}
        </span>
      ))}
      {chipLabels.map((text, i) => (
        <span
          key={i}
          ref={(el) => {
            chipRefs.current[i] = el;
          }}
          style={{ display: "inline-block", fontSize: chipEm, ...typeStyle }}
        >
          {text}
        </span>
      ))}
    </div>
  );
}

export function StatusCycle({ theme, ...settings }: ElementSettings["status-cycle"]) {
  if (!isTheme(theme)) return <StatusCycleView {...settings} />;
  return (
    <WarmUIProvider mode={theme.mode}>
      <StatusCycleView {...settings} colors={themeColors(theme)} mode={theme.mode} />
    </WarmUIProvider>
  );
}
