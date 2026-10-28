import type React from "react";
import { AbsoluteFill, Easing, getRemotionEnvironment, interpolate } from "remotion";
import { type ElementProps } from "#video/common/spec.ts";
import { isTheme, themeColors } from "#video/common/theme.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import {
  Item,
  mixOklch,
  resolveFont,
  type WarmTheme,
  useWarmTheme,
  withAlpha,
} from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

const SANS = '"Inter"';

const REF_W = 810;
const REF_H = 458;

const WORD_DUR = 0.14;
const WORD_SCALE = 1.216;
const WORD_BLUR = 4.3;
const EXIT_DUR = 0.34;
const EXIT_SCALE = 0.8;
const EXIT_BLUR = 6.5;

const WORD_EASE = Easing.out(Easing.quad);

export function pullbackScale(t: number, rate: number, floor = 0.02): number {
  return Math.max(floor, 1 - rate * t);
}

export interface DriftTile {
  glyph: string;
  label?: string;
  background: string;
  color?: string;
  x: number;
  y: number;
  size: number;
  vx?: number;
  vy?: number;
  at?: number;
  radius?: number;
}

export interface PlacedTile {
  left: number;
  top: number;
  size: number;
  opacity: number;
}

export function placeTile(
  tile: DriftTile,
  t: number,
  o: { rate: number; fade: number; speed: number; scale: number },
): PlacedTile {
  const k = pullbackScale(t, o.rate);
  const wx = tile.x + (tile.vx ?? 0) * o.speed * t;
  const wy = tile.y + (tile.vy ?? 0) * o.speed * t;
  const size = k * tile.size * o.scale;
  const age = t - (tile.at ?? 0);
  const opacity = age <= 0 ? 0 : 1 - Math.exp(-age / Math.max(1e-3, o.fade));
  return {
    left: REF_W / 2 + k * wx - size / 2,
    top: REF_H / 2 + k * wy - size / 2,
    size,
    opacity,
  };
}

export interface WordState {
  scale: number;
  blur: number;
  opacity: number;
}

export function wordState(
  t: number,
  index: number,
  o: {
    at: number;
    stagger: number;
    dur: number;
    scale: number;
    blur: number;
    exitAt: number;
    exitDur: number;
    exitScale: number;
    exitBlur: number;
  },
): WordState {
  const start = o.at + index * o.stagger;
  if (t < start) return { scale: o.scale, blur: o.blur, opacity: 0 };
  const p = interpolate(t, [start, start + o.dur], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: WORD_EASE,
  });
  const q = interpolate(t, [o.exitAt, o.exitAt + o.exitDur], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  return {
    scale: (o.scale + (1 - o.scale) * p) * (1 + (o.exitScale - 1) * q),
    blur: o.blur * (1 - p) + o.exitBlur * q,
    opacity: 1 - q,
  };
}

export const WARM_STACK: DriftTile[] = [
  {
    glyph: "Rm",
    label: "Remotion",
    background: "#1f2430",
    color: "#ffffff",
    x: 163,
    y: 232,
    vx: -54,
    vy: 12,
    size: 109,
    at: 0.28,
  },
  {
    glyph: "sh",
    label: "shadcn/ui",
    background: "#f2f4f7",
    color: "#16181d",
    x: 128,
    y: -209,
    vx: 81,
    vy: 32,
    size: 122,
    at: 0.48,
  },
  {
    glyph: "Re",
    label: "React",
    background: "#10131a",
    color: "#7dd3fc",
    x: -384,
    y: 344,
    vx: -27,
    vy: -68,
    size: 200,
    at: 0.95,
  },
  {
    glyph: "N",
    label: "Next.js",
    background: "#e7e8ea",
    color: "#16181d",
    x: 519,
    y: 45,
    vx: -31,
    vy: 64,
    size: 144,
    at: 1.51,
  },
  {
    glyph: "tw",
    label: "Tailwind",
    background: "#f1ece2",
    color: "#8a5a2b",
    x: -692,
    y: -125,
    vx: 41,
    vy: -71,
    size: 214,
    at: 1.55,
  },
  {
    glyph: "Fg",
    label: "Figma",
    background: "#2b3a36",
    color: "#ffffff",
    x: -350,
    y: -339,
    vx: 65,
    vy: -26,
    size: 188,
    at: 1.75,
  },
  {
    glyph: "Mo",
    label: "Motion",
    background: "#f4703a",
    color: "#ffffff",
    x: 672,
    y: -277,
    vx: 2,
    vy: 81,
    size: 256,
    at: 1.76,
  },
  {
    glyph: "TS",
    label: "TypeScript",
    background: "#2f6fdb",
    color: "#ffffff",
    x: -721,
    y: 211,
    vx: 9,
    vy: -84,
    size: 215,
    at: 2.06,
  },
  {
    glyph: "▲",
    label: "Vercel",
    background: "#0b0b0d",
    color: "#ffffff",
    x: 676,
    y: 348,
    vx: -65,
    vy: 52,
    size: 244,
    at: 2.48,
  },
];

export interface LogoDriftProps {
  headline?: string;
  tiles?: DriftTile[];

  pullback?: number;
  tileSpeed?: number;
  tileScale?: number;
  tileFade?: number;
  tileRadius?: number;

  wordAt?: number;
  wordStagger?: number;
  wordDuration?: number;
  wordScale?: number;
  wordBlur?: number;

  exitAt?: number;
  exitDuration?: number;
  exitScale?: number;
  exitBlur?: number;

  fontSize?: number;
  fontWeight?: number;
  glyphScale?: number;
  glow?: boolean;
  glowColor?: string;
  glowOpacity?: number;
  accentColor?: string;
  theme?: Partial<WarmTheme>;
  mode?: "light" | "dark";
  fontFamily?: string;
  speed?: number;
}

function LogoDriftView({
  headline = "Built for the stack you already use.",
  tiles = WARM_STACK,
  pullback = 0.1762,
  tileSpeed = 1,
  tileScale = 1,
  tileFade = 0.4,
  tileRadius = 0.02,
  wordAt = 0.017,
  wordStagger = 0.15,
  wordDuration = WORD_DUR,
  wordScale = WORD_SCALE,
  wordBlur = WORD_BLUR,
  exitAt = 3.13,
  exitDuration = EXIT_DUR,
  exitScale = EXIT_SCALE,
  exitBlur = EXIT_BLUR,
  fontSize = 20,
  fontWeight = 500,
  glyphScale = 0.34,
  glow = true,
  glowColor,
  glowOpacity = 0.07,
  accentColor,
  theme,
  mode,
  fontFamily,
  speed = 1,
}: LogoDriftProps) {
  const frame = useAuthoredFrame();
  const { width, height, fps } = useAuthoredConfig();
  const t = useWarmTheme(theme, mode);
  const face = resolveFont(fontFamily ?? t.fontFamily) ?? SANS;
  const accent = accentColor ?? t.primary;
  const wash = glowColor ?? accent;

  const now = (frame * speed) / fps;
  const stageScale = Math.min(width / REF_W, height / REF_H);

  const words = headline.split(/\s+/).filter(Boolean);
  const isRendering = getRemotionEnvironment().isRendering;
  const willChange = isRendering ? undefined : ("transform, filter" as const);

  return (
    <AbsoluteFill style={{ background: t.background }}>
      <div
        style={{
          position: "absolute",
          left: "50%",
          top: "50%",
          width: REF_W,
          height: REF_H,
          transform: `translate(-50%, -50%) scale(${stageScale})`,
          overflow: "hidden",
          background: t.background,
        }}
      >
        {glow ? (
          <div
            style={{
              position: "absolute",
              left: 0,
              top: 0,
              width: REF_W,
              height: REF_H,
              background: `radial-gradient(${0.45 * REF_W}px ${0.34 * REF_H}px at 50% 49%, ${withAlpha(wash, glowOpacity)} 0%, transparent 74%)`,
            }}
          />
        ) : null}

        {tiles.map((tile, i) => {
          const p = placeTile(tile, now, {
            rate: pullback,
            fade: tileFade,
            speed: tileSpeed,
            scale: tileScale,
          });
          if (p.opacity <= 0.002 || p.size < 1) return null;
          const fg = tile.color ?? t.background;
          return (
            <Item key={`${tile.glyph}-${tile.x}-${tile.y}`} index={i}>
              <div
                style={{
                  position: "absolute",
                  left: p.left,
                  top: p.top,
                  width: p.size,
                  height: p.size,
                  borderRadius: p.size * (tile.radius ?? tileRadius),
                  background: tile.background,
                  opacity: p.opacity,
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: p.size * 0.04,
                  color: fg,
                  fontFamily: face,
                  overflow: "hidden",
                  willChange,
                }}
              >
                <span
                  style={{
                    fontSize: p.size * glyphScale,
                    fontWeight: 600,
                    lineHeight: 1,
                    letterSpacing: "-0.03em",
                    textRendering: "geometricPrecision",
                  }}
                >
                  {tile.glyph}
                </span>
                {tile.label && p.size > 74 ? (
                  <span
                    style={{
                      fontSize: p.size * 0.1,
                      fontWeight: 500,
                      lineHeight: 1,
                      opacity: 0.72,
                      textRendering: "geometricPrecision",
                    }}
                  >
                    {tile.label}
                  </span>
                ) : null}
              </div>
            </Item>
          );
        })}

        <div
          style={{
            position: "absolute",
            left: 0,
            top: 0,
            width: REF_W,
            height: REF_H,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              maxWidth: REF_W * 0.88,
              alignItems: "baseline",
              justifyContent: "center",
              columnGap: fontSize * 0.28,
              rowGap: fontSize * 0.08,
              fontFamily: face,
              fontSize,
              fontWeight,
              lineHeight: 1.2,
              letterSpacing: "-0.012em",
              color: t.foreground,
              whiteSpace: "pre",
              textRendering: "geometricPrecision",
            }}
          >
            {words.map((word, i) => {
              const w = wordState(now, i, {
                at: wordAt,
                stagger: wordStagger,
                dur: wordDuration,
                scale: wordScale,
                blur: wordBlur,
                exitAt,
                exitDur: exitDuration,
                exitScale,
                exitBlur,
              });
              return (
                <span
                  key={`${word}-${i}`}
                  style={{
                    display: "inline-block",
                    opacity: w.opacity,
                    transform: `scale(${w.scale})`,
                    filter: w.blur > 0.02 ? `blur(${w.blur}px)` : undefined,
                    willChange,
                  }}
                >
                  {word}
                </span>
              );
            })}
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}

export const driftTileTint = (t: WarmTheme, amount: number): string =>
  mixOklch(t.card, t.primary, amount);

export function LogoDrift(props: ElementProps & { children?: React.ReactNode }) {
  const { theme } = props;
  if (!isTheme(theme))
    return <LogoDriftView {...(props as unknown as React.ComponentProps<typeof LogoDriftView>)} />;
  return (
    <WarmUIProvider mode={theme.mode}>
      <LogoDriftView
        {...(props as unknown as React.ComponentProps<typeof LogoDriftView>)}
        theme={themeColors(theme)}
        mode={theme.mode}
      />
    </WarmUIProvider>
  );
}
