import { type CSSProperties, useCallback, useState } from "react";
import type React from "react";
import {
  AbsoluteFill,
  continueRender,
  delayRender,
  Easing,
  getRemotionEnvironment,
  interpolate,
  OffthreadVideo,
  staticFile,
} from "remotion";
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

export interface HeroLaunchProps {
  heading?: string;
  image1?: string;
  image2?: string;
  theme?: Partial<WarmTheme>;
  mode?: "light" | "dark";
  fontFamily?: string;
}

const PERSP = 1000;
const TILT = 17;
const GAP = 92;
const placeholder = (t: WarmTheme) =>
  `linear-gradient(135deg,${mixOklch(t.background, t.card, 0.55)},${mixOklch(
    t.background,
    t.card,
    0.2,
  )})`;

const isVideo = (src: string) => /\.(mp4|webm|mov|m4v)(\?|$)/i.test(src);
const isGradient = (src: string) => src.startsWith("linear-gradient");

const OUT = Easing.bezier(0.22, 1, 0.36, 1);
const FALL = Easing.bezier(0.5, 0, 0.9, 0.4);

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

function HeroImage({ src, style }: { src: string; style: CSSProperties }) {
  const [handle] = useState(() => delayRender(`hero image: ${src}`));
  const release = useCallback(() => continueRender(handle), [handle]);
  return <img src={resolveSrc(src)} alt="" style={style} onLoad={release} onError={release} />;
}

function CardMedia({ src, t }: { src: string; t: WarmTheme }) {
  const media: CSSProperties = {
    position: "absolute",
    inset: 0,
    width: "100%",
    height: "100%",
    objectFit: "cover",
  };
  const effective = src || placeholder(t);
  if (isGradient(effective)) {
    return <div style={{ ...media, background: effective }} />;
  }
  if (isVideo(effective)) {
    return <OffthreadVideo src={resolveSrc(effective)} muted style={media} />;
  }
  return <HeroImage src={effective} style={media} />;
}

function HeroLaunchView({
  heading = "Launch day, done right",
  image1 = "",
  image2 = "",
  theme,
  mode,
  fontFamily,
}: HeroLaunchProps) {
  const frame = useAuthoredFrame();
  const { width, height } = useAuthoredConfig();
  const t = useWarmTheme(theme, mode ?? "dark");
  const face = resolveFont(fontFamily ?? t.fontFamily) ?? SANS;
  const s = Math.min(height / 720, width / 1280);

  const cardW = 1000 * s;
  const cardH = 620 * s;
  const centerLeft = (width - cardW) / 2;
  const cardTop = 0.3 * height;
  const restOffset = (cardW + GAP * s) / 2;
  const leftXFinal = -restOffset;
  const rightXFinal = restOffset;
  const rightXStart = 1.08 * width;
  const groupYFinal = 0.16 * height;

  const seg = (start: number, dur: number, easing = OUT) =>
    interpolate(frame, [start, start + dur], [0, 1], {
      easing,
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    });

  const recede = seg(40, 30);
  const angle = seg(46, 36);
  const slideL = seg(46, 36);
  const slideR = seg(60, 40);
  const drop = seg(122, 34, FALL);
  const title = seg(126, 40);

  const leftScale = interpolate(recede, [0, 1], [1.5, 1]);
  const leftX = interpolate(slideL, [0, 1], [0, leftXFinal]);
  const leftDeg = interpolate(angle, [0, 1], [0, -TILT]);
  const leftOpacity = interpolate(frame, [2, 12], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const rightX = interpolate(slideR, [0, 1], [rightXStart, rightXFinal]);
  const rightOpacity = interpolate(slideR, [0, 0.35], [0, 1], {
    extrapolateRight: "clamp",
  });

  const floatEnv = seg(94, 22) * (1 - drop);
  const floatY = Math.sin(frame * 0.09) * 3.5 * s * floatEnv;
  const groupY = interpolate(drop, [0, 1], [0, groupYFinal]) + floatY;

  const titleY = interpolate(title, [0, 1], [40 * s, 0]);
  const titleScale = interpolate(title, [0, 1], [0.82, 1]);
  const titleBlur = interpolate(title, [0, 1], [10, 0]);

  const vignette = seg(50, 40, Easing.linear);

  const cardBox = (x: number, opacity: number): CSSProperties => ({
    position: "absolute",
    left: centerLeft,
    top: cardTop,
    width: cardW,
    height: cardH,
    opacity,
    transform: `translateX(${x}px)`,
  });

  const zoom = (scale: number): CSSProperties => ({
    width: "100%",
    height: "100%",
    transform: `scale(${scale})`,
    transformOrigin: "center center",
  });
  const tilt = (deg: number, origin: string): CSSProperties => ({
    width: "100%",
    height: "100%",
    transform: `perspective(${PERSP * s}px) rotateY(${deg}deg)`,
    transformOrigin: origin,
  });
  const fade = (gradient: string): CSSProperties => ({
    position: "absolute",
    inset: 0,
    background: gradient,
  });

  return (
    <AbsoluteFill style={{ backgroundColor: t.background, fontFamily: face }}>
      <AbsoluteFill style={{ transform: `translateY(${groupY}px)` }}>
        <Item index={0}>
          <div style={cardBox(leftX, leftOpacity)}>
            <div style={zoom(leftScale)}>
              <div style={tilt(leftDeg, "100% 50%")}>
                <ImageCard s={s} src={image1} t={t} />
              </div>
            </div>
          </div>
        </Item>
        <Item index={1}>
          <div style={cardBox(rightX, rightOpacity)}>
            <div style={tilt(TILT, "0% 50%")}>
              <ImageCard s={s} src={image2} t={t} />
            </div>
          </div>
        </Item>
      </AbsoluteFill>

      <AbsoluteFill style={{ opacity: vignette, pointerEvents: "none" }}>
        <div style={fade(`linear-gradient(to left, ${t.background} 0%, transparent 30%)`)} />
        <div style={fade(`linear-gradient(to right, ${t.background} 0%, transparent 22%)`)} />
        <div style={fade(`linear-gradient(to top, ${t.background} 0%, transparent 30%)`)} />
      </AbsoluteFill>

      <div
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          top: 0.22 * height,
          padding: `0 ${40 * s}px`,
          textAlign: "center",
          color: t.foreground,
          fontWeight: 600,
          letterSpacing: "-0.02em",
          fontSize: 42 * s,
          opacity: title,
          filter: `blur(${titleBlur}px)`,
          transform: `translateY(${titleY}px) scale(${titleScale})`,
        }}
      >
        {heading}
      </div>
    </AbsoluteFill>
  );
}

function ImageCard({ s, src, t }: { s: number; src: string; t: WarmTheme }) {
  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        height: "100%",
        overflow: "hidden",
        borderRadius: 30 * s,
        background: mixOklch(t.background, t.card, 0.35),
        boxShadow: `0 ${20 * s}px ${52 * s}px -${28 * s}px ${withAlpha(
          mixOklch(t.background, "#000", 0.75),
          0.9,
        )}, inset 0 0 0 1px ${withAlpha(t.foreground, 0.22)}`,
      }}
    >
      {src ? <CardMedia src={src} t={t} /> : null}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `linear-gradient(to top, ${withAlpha(
            mixOklch(t.background, "#000", 0.75),
            0.28,
          )} 0%, transparent 32%)`,
        }}
      />
      <div
        style={{
          position: "absolute",
          left: 26 * s,
          right: 26 * s,
          top: 0,
          height: 1,
          background: `linear-gradient(90deg,transparent,${withAlpha(
            t.foreground,
            0.5,
          )},transparent)`,
        }}
      />
    </div>
  );
}

export function HeroLaunch(props: ElementProps & { children?: React.ReactNode }) {
  const { theme } = props;
  if (!isTheme(theme))
    return (
      <HeroLaunchView {...(props as unknown as React.ComponentProps<typeof HeroLaunchView>)} />
    );
  return (
    <WarmUIProvider mode={theme.mode}>
      <HeroLaunchView
        {...(props as unknown as React.ComponentProps<typeof HeroLaunchView>)}
        theme={themeColors(theme)}
        mode={theme.mode}
      />
    </WarmUIProvider>
  );
}
