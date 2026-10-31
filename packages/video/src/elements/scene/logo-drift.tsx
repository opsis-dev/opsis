import { Easing } from "@opsis/runtime/easing.ts";
import { isRendering } from "@opsis/runtime/environment.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { themeColors } from "#video/common/theme.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings, LogoDriftTile } from "#video/generated/spec.ts";
import { withAlpha } from "#video/lib/ui-kit/color.ts";
import { Item, resolveFont, type WarmTheme, useWarmTheme } from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

const SANS = '"Inter"';

const REF_W = 810;
const REF_H = 458;

const WORD_EASE = Easing.out(Easing.quad);

export function pullbackScale(t: number, rate: number, floor = 0.02): number {
  return Math.max(floor, 1 - rate * t);
}

export interface PlacedTile {
  left: number;
  top: number;
  size: number;
  opacity: number;
}

export function placeTile(
  tile: LogoDriftTile,
  t: number,
  o: { rate: number; fade: number; speed: number; scale: number },
): PlacedTile {
  const k = pullbackScale(t, o.rate);
  const wx = tile.x + tile.vx * o.speed * t;
  const wy = tile.y + tile.vy * o.speed * t;
  const size = k * tile.size * o.scale;
  const age = t - tile.at;
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
  const p = interpolate(t, [start, start + o.dur], [0, 1], { ...CLAMPED, easing: WORD_EASE });
  const q = interpolate(t, [o.exitAt, o.exitAt + o.exitDur], [0, 1], CLAMPED);
  return {
    scale: (o.scale + (1 - o.scale) * p) * (1 + (o.exitScale - 1) * q),
    blur: o.blur * (1 - p) + o.exitBlur * q,
    opacity: 1 - q,
  };
}

type LogoDriftProps = Omit<ElementSettings["logo-drift"], "theme"> & {
  theme: Partial<WarmTheme>;
  mode: "light" | "dark";
};

function LogoDriftView({
  headline,
  tiles,
  pullback,
  tileSpeed,
  tileScale,
  tileFade,
  tileRadius,
  wordAt,
  wordStagger,
  wordDuration,
  wordScale,
  wordBlur,
  exitAt,
  exitDuration,
  exitScale,
  exitBlur,
  fontSize,
  fontWeight,
  glyphScale,
  glow,
  glowColor,
  glowOpacity,
  accentColor,
  theme,
  mode,
  fontFamily,
  speed,
  glowSize,
  glyphWeight,
  labelScale,
  labelOpacity,
  headlineWidth,
}: LogoDriftProps) {
  const frame = useAuthoredFrame();
  const { width, height, fps } = useAuthoredConfig();
  const t = useWarmTheme(theme, mode);
  const face = resolveFont(fontFamily || t.fontFamily) ?? SANS;
  const accent = accentColor || t.primary;
  const wash = glowColor || accent;

  const now = (frame * speed) / fps;
  const stageScale = Math.min(width / REF_W, height / REF_H);

  const words = headline.split(/\s+/).filter(Boolean);
  const rendering = isRendering();
  const willChange = rendering ? undefined : ("transform, filter" as const);

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
              background: `radial-gradient(${0.45 * glowSize * REF_W}px ${0.34 * glowSize * REF_H}px at 50% 49%, ${withAlpha(wash, glowOpacity)} 0%, transparent 74%)`,
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
          const fg = tile.color || t.background;
          return (
            <Item key={`${tile.glyph}-${tile.x}-${tile.y}`} index={i}>
              <div
                style={{
                  position: "absolute",
                  left: p.left,
                  top: p.top,
                  width: p.size,
                  height: p.size,
                  borderRadius: p.size * tileRadius,
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
                    fontWeight: glyphWeight,
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
                      fontSize: p.size * labelScale,
                      fontWeight: 500,
                      lineHeight: 1,
                      opacity: labelOpacity,
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
              maxWidth: (REF_W * headlineWidth) / 100,
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

export function LogoDrift({ theme, ...settings }: ElementSettings["logo-drift"]) {
  return (
    <WarmUIProvider mode={theme.mode}>
      <LogoDriftView {...settings} theme={themeColors(theme)} mode={theme.mode} />
    </WarmUIProvider>
  );
}
