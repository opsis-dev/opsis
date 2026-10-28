import type React from "react";
import { Easing, getRemotionEnvironment, Img, interpolate, staticFile } from "remotion";
import type { ElementProps } from "#video/common/spec.ts";
import { isThemeValue, themeColors } from "#video/common/theme.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import { listsFrom } from "#video/lib/settings/shape.ts";
import {
  Item,
  mixOklch,
  resolveFont,
  type WarmTheme,
  useWarmTheme,
  withAlpha,
} from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

export interface LogoAssembleProps {
  logoSrc?: string;
  brandName?: string;
  middleText?: string;
  images?: string[];
  count?: number;
  background?: string;
  speed?: number;
  className?: string;
  theme?: Partial<WarmTheme>;
  mode?: "light" | "dark";
  fontFamily?: string;
}

const FONT_FAMILY = 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';

const DEFAULT_IMAGES: string[] = [];

export const CONTRACT_START = 50;
export const CONTRACT_END = 66;
export const MIDDLE_OUT = 52;
export const LOGO_IN = 58;
export const LOGO_SHIFT_START = 74;
export const LOGO_SHIFT_END = 90;
export const NAME_IN = LOGO_SHIFT_START;

export const ORBIT_SPEED = 0.1;
export const DRAIN_SPIN = Math.PI * 1.1;
const CARD_RATIO = 1.22;

export const EASE = Easing.bezier(0.2, 0.6, 0.35, 1);
export const EASE_IN = Easing.bezier(0.5, 0, 0.75, 0.4);
const CLAMP = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

export function contractProgress(frame: number): number {
  return interpolate(frame, [CONTRACT_START, CONTRACT_END], [0, 1], {
    ...CLAMP,
    easing: EASE_IN,
  });
}

export function orbitAngle(frame: number): number {
  return frame * ORBIT_SPEED + contractProgress(frame) * DRAIN_SPIN;
}

export function ringScale(frame: number): number {
  return 1 - contractProgress(frame);
}

export function cardScale(frame: number): number {
  return 1 - 0.66 * contractProgress(frame);
}

export function cardOpacity(frame: number): number {
  return interpolate(frame, [CONTRACT_END - 8, CONTRACT_END], [1, 0], CLAMP);
}

export function middleTextOpacity(frame: number): number {
  return interpolate(frame, [6, 16, MIDDLE_OUT - 6, MIDDLE_OUT], [0, 1, 1, 0], CLAMP);
}

export interface LogoPose {
  opacity: number;
  scale: number;
}

export function logoPose(frame: number): LogoPose {
  return {
    opacity: interpolate(frame, [LOGO_IN, LOGO_IN + 8], [0, 1], CLAMP),
    scale: interpolate(frame, [LOGO_IN, LOGO_IN + 10, LOGO_IN + 18], [0.6, 1.05, 1], {
      ...CLAMP,
      easing: EASE,
    }),
  };
}

export function logoShift(frame: number): number {
  return interpolate(frame, [LOGO_SHIFT_START, LOGO_SHIFT_END], [0, 1], {
    ...CLAMP,
    easing: EASE,
  });
}

export interface NamePose {
  opacity: number;
  dx: number;
}

export function namePose(frame: number): NamePose {
  return {
    opacity: interpolate(frame, [NAME_IN, NAME_IN + 10], [0, 1], CLAMP),
    dx: interpolate(frame, [NAME_IN, NAME_IN + 12], [18, 0], {
      ...CLAMP,
      easing: EASE,
    }),
  };
}

function resolveSrc(src: string): string {
  const isLocal = src.startsWith("/") && !src.startsWith("//");
  if (!isLocal || getRemotionEnvironment().isPlayer) return src;
  const base = staticFile("_").slice(0, -2);
  if (base && src.startsWith(`${base}/`)) return src;
  try {
    return staticFile(src.replace(/^\/+/, ""));
  } catch {
    return src;
  }
}

function LogoAssembleView({
  logoSrc = "",
  brandName = "Acme",
  middleText = "Cinematic components\nfor React",
  images: imageList = DEFAULT_IMAGES,
  count = 10,
  background,
  speed = 1,
  className,
  theme,
  mode,
  fontFamily,
}: LogoAssembleProps) {
  const images = imageList.filter((src) => src.trim() !== "");
  const frame = useAuthoredFrame() * speed;
  const { width, height } = useAuthoredConfig();
  const t = useWarmTheme(theme, mode ?? "dark");
  const face = resolveFont(fontFamily ?? t.fontFamily) ?? FONT_FAMILY;
  const stage = background ?? t.background;
  const cardShadow = withAlpha(mixOklch(stage, "#000", 0.75), 0.4);
  const cx = width / 2;
  const cy = height / 2;
  const short = Math.min(width, height);
  const isRendering = getRemotionEnvironment().isRendering;

  const R = short * 0.36;
  const cardW = Math.round(short * 0.15);
  const cardH = cardW * CARD_RATIO;

  const a = orbitAngle(frame);
  const rs = ringScale(frame);
  const cScale = cardScale(frame);
  const cOpacity = cardOpacity(frame);
  const middleOp = middleTextOpacity(frame);
  const logo = logoPose(frame);
  const shift = logoShift(frame);
  const name = namePose(frame);
  const n = Math.max(1, Math.floor(count));

  const logoSize = Math.round(short * 0.125);
  const gap = Math.round(short * 0.035);
  const restX = short * 0.13;
  const logoCX = cx - restX * shift;
  const nameLeft = cx - restX + logoSize / 2 + gap;
  const nameSize = Math.round(short * 0.055);

  return (
    <div
      className={className}
      style={{
        position: "absolute",
        inset: 0,
        overflow: "hidden",
        backgroundColor: stage,
        fontFamily: face,
        textRendering: "geometricPrecision",
      }}
    >
      {cOpacity > 0 &&
        images.length > 0 &&
        Array.from({ length: n }, (_, i) => {
          const theta = (i / n) * Math.PI * 2 + a;
          const x = cx + Math.cos(theta) * R * rs;
          const y = cy + Math.sin(theta) * R * rs;
          const k = i % images.length;
          return (
            <Item key={`card-${i}`} index={k} primary={i === k}>
              <div
                style={{
                  position: "absolute",
                  left: x - cardW / 2,
                  top: y - cardH / 2,
                  width: cardW,
                  height: cardH,
                  opacity: cOpacity,
                  transform: `scale(${cScale})`,
                  overflow: "hidden",
                  borderRadius: 4,
                  boxShadow: `0 14px 34px ${cardShadow}`,
                  ...(isRendering ? {} : { willChange: "transform" as const }),
                }}
              >
                <Img
                  src={resolveSrc(images[k] ?? images[0])}
                  style={{
                    position: "absolute",
                    inset: 0,
                    width: "100%",
                    height: "100%",
                    maxWidth: "none",
                    objectFit: "cover",
                  }}
                />
              </div>
            </Item>
          );
        })}

      {middleOp > 0 && middleText && (
        <div
          style={{
            position: "absolute",
            left: cx - R,
            top: cy - cardH,
            width: R * 2,
            height: cardH * 2,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            textAlign: "center",
            opacity: middleOp,
            color: t.foreground,
            fontSize: Math.round(short * 0.04),
            fontWeight: 600,
            lineHeight: 1.2,
            letterSpacing: "-0.02em",
            whiteSpace: "pre-line",
          }}
        >
          {middleText}
        </div>
      )}

      {logo.opacity > 0 && logoSrc.trim() !== "" && (
        <Img
          src={resolveSrc(logoSrc)}
          style={{
            position: "absolute",
            left: logoCX - logoSize / 2,
            top: cy - logoSize / 2,
            width: logoSize,
            height: logoSize,
            maxWidth: "none",
            objectFit: "contain",
            opacity: logo.opacity,
            transform: `scale(${logo.scale})`,
            transformOrigin: "center center",
          }}
        />
      )}

      {name.opacity > 0 && (
        <div
          style={{
            position: "absolute",
            left: nameLeft,
            top: cy - nameSize,
            height: nameSize * 2,
            display: "flex",
            alignItems: "center",
            opacity: name.opacity,
            transform: `translateX(${name.dx}px)`,
            color: t.foreground,
            fontSize: nameSize,
            fontWeight: 600,
            letterSpacing: "-0.02em",
            whiteSpace: "nowrap",
          }}
        >
          {brandName}
        </div>
      )}
    </div>
  );
}

type LogoAssembleViewProps = React.ComponentProps<typeof LogoAssembleView>;

export function LogoAssemble(props: ElementProps) {
  const { theme } = props;
  if (!isThemeValue(theme))
    return (
      <LogoAssembleView
        {...(listsFrom(props, { images: { image: "images" } }) as unknown as LogoAssembleViewProps)}
      />
    );
  const themed = { ...props, theme: themeColors(theme), mode: theme.mode };
  return (
    <WarmUIProvider mode={theme.mode}>
      <LogoAssembleView
        {...(listsFrom(themed, {
          images: { image: "images" },
        }) as unknown as LogoAssembleViewProps)}
      />
    </WarmUIProvider>
  );
}
