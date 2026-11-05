import { Easing } from "@opsis/runtime/easing.ts";
import { isRendering } from "@opsis/runtime/environment.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { Img } from "@opsis/runtime/parts.tsx";
import { themeColors } from "#video/common/theme.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { mixOklch, withAlpha } from "#video/lib/ui-kit/color.ts";
import { Item, resolveFont, type WarmTheme, useWarmTheme } from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

type LogoAssembleProps = Omit<ElementSettings["logo-assemble"], "theme"> & {
  theme: Partial<WarmTheme>;
  mode: "light" | "dark";
};

const FONT_FAMILY = 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';

type Beats = {
  contractStart: number;
  contractEnd: number;
  middleOut: number;
  logoIn: number;
  logoShiftStart: number;
  logoShiftEnd: number;
  nameIn: number;
};

function beatsFrom(contractStart: number): Beats {
  return {
    contractStart,
    contractEnd: contractStart + 16,
    middleOut: contractStart + 2,
    logoIn: contractStart + 8,
    logoShiftStart: contractStart + 24,
    logoShiftEnd: contractStart + 40,
    nameIn: contractStart + 24,
  };
}

const ORBIT_SPEED = 0.1;
const DRAIN_SPIN = Math.PI * 1.1;

const EASE = Easing.bezier(0.2, 0.6, 0.35, 1);
const EASE_IN = Easing.bezier(0.5, 0, 0.75, 0.4);
function contractProgress(frame: number, b: Beats): number {
  return interpolate(frame, [b.contractStart, b.contractEnd], [0, 1], {
    ...CLAMPED,
    easing: EASE_IN,
  });
}

function orbitAngle(frame: number, b: Beats, speed: number): number {
  return frame * ORBIT_SPEED * speed + contractProgress(frame, b) * DRAIN_SPIN;
}

function ringScale(frame: number, b: Beats): number {
  return 1 - contractProgress(frame, b);
}

function cardScale(frame: number, b: Beats): number {
  return 1 - 0.66 * contractProgress(frame, b);
}

function cardOpacity(frame: number, b: Beats): number {
  return interpolate(frame, [b.contractEnd - 8, b.contractEnd], [1, 0], CLAMPED);
}

function middleTextOpacity(frame: number, b: Beats): number {
  const fadeIn = Math.min(16, b.middleOut - 6);
  return interpolate(
    frame,
    [Math.min(6, fadeIn - 1), fadeIn, b.middleOut - 6, b.middleOut],
    [0, 1, 1, 0],
    CLAMPED,
  );
}

interface LogoPose {
  opacity: number;
  scale: number;
}

function logoPose(frame: number, b: Beats): LogoPose {
  return {
    opacity: interpolate(frame, [b.logoIn, b.logoIn + 8], [0, 1], CLAMPED),
    scale: interpolate(frame, [b.logoIn, b.logoIn + 10, b.logoIn + 18], [0.6, 1.05, 1], {
      ...CLAMPED,
      easing: EASE,
    }),
  };
}

function logoShift(frame: number, b: Beats): number {
  return interpolate(frame, [b.logoShiftStart, b.logoShiftEnd], [0, 1], {
    ...CLAMPED,
    easing: EASE,
  });
}

interface NamePose {
  opacity: number;
  dx: number;
}

function namePose(frame: number, b: Beats): NamePose {
  return {
    opacity: interpolate(frame, [b.nameIn, b.nameIn + 10], [0, 1], CLAMPED),
    dx: interpolate(frame, [b.nameIn, b.nameIn + 12], [18, 0], {
      ...CLAMPED,
      easing: EASE,
    }),
  };
}

function LogoAssembleView({
  logoSrc,
  brandName,
  middleText,
  images: imageList,
  count,
  background,
  speed,
  theme,
  mode,
  fontFamily,
  orbitSeconds,
  orbitSpeed,
  cardRatio,
  cardShadow: shadowAlpha,
  cardRadius,
  ringSize,
  cardSize,
  logoSize: logoShare,
  logoGap,
  nameSize: nameShare,
  middleTextSize,
  fontWeight,
}: LogoAssembleProps) {
  const images = imageList.map((row) => row.image).filter((src) => src.trim() !== "");
  const frame = useAuthoredFrame() * speed;
  const { width, height } = useAuthoredConfig();
  const t = useWarmTheme(theme, mode);
  const face = resolveFont(fontFamily || t.fontFamily) ?? FONT_FAMILY;
  const stage = background || t.background;
  const cardShadow = withAlpha(mixOklch(stage, "#000", 0.75), shadowAlpha);
  const beats = beatsFrom(orbitSeconds);
  const cx = width / 2;
  const cy = height / 2;
  const short = Math.min(width, height);
  const rendering = isRendering();

  const R = (short * ringSize) / 100;
  const cardW = Math.round((short * cardSize) / 100);
  const cardH = cardW * cardRatio;

  const a = orbitAngle(frame, beats, orbitSpeed);
  const rs = ringScale(frame, beats);
  const cScale = cardScale(frame, beats);
  const cOpacity = cardOpacity(frame, beats);
  const middleOp = middleTextOpacity(frame, beats);
  const logo = logoPose(frame, beats);
  const shift = logoShift(frame, beats);
  const name = namePose(frame, beats);
  const n = Math.max(1, Math.floor(count));

  const logoSize = Math.round((short * logoShare) / 100);
  const gap = Math.round((short * logoGap) / 100);
  const restX = short * 0.13;
  const logoCX = cx - restX * shift;
  const nameLeft = cx - restX + logoSize / 2 + gap;
  const nameSize = Math.round((short * nameShare) / 100);

  return (
    <div
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
                  borderRadius: cardRadius,
                  boxShadow: `0 14px 34px ${cardShadow}`,
                  ...(rendering ? {} : { willChange: "transform" as const }),
                }}
              >
                <Img
                  src={images[k] ?? images[0]}
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
            fontSize: Math.round((short * middleTextSize) / 100),
            fontWeight,
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
          src={logoSrc}
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
            fontWeight,
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

export function LogoAssemble({ theme, ...settings }: ElementSettings["logo-assemble"]) {
  return (
    <WarmUIProvider mode={theme.mode}>
      <LogoAssembleView {...settings} theme={themeColors(theme)} mode={theme.mode} />
    </WarmUIProvider>
  );
}
