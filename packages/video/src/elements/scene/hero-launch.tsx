import { type CSSProperties, useCallback, useState } from "react";
import {
  AbsoluteFill,
  continueRender,
  delayRender,
  Easing,
  interpolate,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { themeColors } from "#video/common/theme.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import { FootageFrame, type FootageUrls } from "#video/core/footage.tsx";
import type { ElementSettings, MediaKind } from "#video/generated/spec.ts";
import { mixOklch, withAlpha } from "#video/lib/ui-kit/color.ts";
import { Item, resolveFont, type WarmTheme, useWarmTheme } from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

const SANS = '"Inter"';

type HeroLaunchProps = Omit<ElementSettings["hero-launch"], "theme"> & {
  image1Kind?: MediaKind;
  image2Kind?: MediaKind;
  footageUrls?: FootageUrls;
  theme: Partial<WarmTheme>;
  mode: "light" | "dark";
};

const PERSP = 1000;

const OUT = Easing.bezier(0.22, 1, 0.36, 1);
const FALL = Easing.bezier(0.5, 0, 0.9, 0.4);

function HeroImage({ src, style }: { src: string; style: CSSProperties }) {
  const [handle] = useState(() => delayRender(`hero image: ${src}`));
  const release = useCallback(() => continueRender(handle), [handle]);
  return <img src={src} alt="" style={style} onLoad={release} onError={release} />;
}

function CardMedia({
  src,
  kind,
  footage,
}: {
  src: string;
  kind: MediaKind | undefined;
  footage?: string;
}) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const media: CSSProperties = {
    position: "absolute",
    inset: 0,
    width: "100%",
    height: "100%",
    objectFit: "cover",
  };
  if (kind === "video") {
    return footage ? (
      <FootageFrame url={footage} timeS={frame / fps} fit="cover" style={media} />
    ) : null;
  }
  return <HeroImage src={src} style={media} />;
}

function HeroLaunchView({
  heading,
  image1,
  image1Kind,
  image2,
  image2Kind,
  footageUrls,
  theme,
  mode,
  fontFamily,
  cardTilt,
  cardGap,
  cardAspect,
  cardRadius,
  headlineSize,
  headlineWeight,
  edgeFade,
}: HeroLaunchProps) {
  const frame = useAuthoredFrame();
  const { width, height } = useAuthoredConfig();
  const t = useWarmTheme(theme, mode);
  const face = resolveFont(fontFamily || t.fontFamily) ?? SANS;
  const s = Math.min(height / 720, width / 1280);

  const cardW = 1000 * s;
  const cardH = cardW / cardAspect;
  const centerLeft = (width - cardW) / 2;
  const cardTop = 0.3 * height;
  const restOffset = (cardW + cardGap * s) / 2;
  const leftXFinal = -restOffset;
  const rightXFinal = restOffset;
  const rightXStart = 1.08 * width;
  const groupYFinal = 0.16 * height;

  function seg(start: number, dur: number, easing = OUT) {
    return interpolate(frame, [start, start + dur], [0, 1], { ...CLAMPED, easing });
  }

  const recede = seg(40, 30);
  const angle = seg(46, 36);
  const slideL = seg(46, 36);
  const slideR = seg(60, 40);
  const drop = seg(122, 34, FALL);
  const title = seg(126, 40);

  const leftScale = interpolate(recede, [0, 1], [1.5, 1]);
  const leftX = interpolate(slideL, [0, 1], [0, leftXFinal]);
  const leftDeg = interpolate(angle, [0, 1], [0, -cardTilt]);
  const leftOpacity = interpolate(frame, [2, 12], [0, 1], CLAMPED);

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

  function cardBox(x: number, opacity: number): CSSProperties {
    return {
      position: "absolute",
      left: centerLeft,
      top: cardTop,
      width: cardW,
      height: cardH,
      opacity,
      transform: `translateX(${x}px)`,
    };
  }

  function zoom(scale: number): CSSProperties {
    return {
      width: "100%",
      height: "100%",
      transform: `scale(${scale})`,
      transformOrigin: "center center",
    };
  }
  function tilt(deg: number, origin: string): CSSProperties {
    return {
      width: "100%",
      height: "100%",
      transform: `perspective(${PERSP * s}px) rotateY(${deg}deg)`,
      transformOrigin: origin,
    };
  }
  function fade(gradient: string): CSSProperties {
    return {
      position: "absolute",
      inset: 0,
      background: gradient,
    };
  }

  return (
    <AbsoluteFill style={{ backgroundColor: t.background, fontFamily: face }}>
      <AbsoluteFill style={{ transform: `translateY(${groupY}px)` }}>
        <Item index={0}>
          <div style={cardBox(leftX, leftOpacity)}>
            <div style={zoom(leftScale)}>
              <div style={tilt(leftDeg, "100% 50%")}>
                <ImageCard
                  s={s}
                  src={image1}
                  kind={image1Kind}
                  footage={footageUrls?.image1}
                  t={t}
                  radius={cardRadius}
                />
              </div>
            </div>
          </div>
        </Item>
        <Item index={1}>
          <div style={cardBox(rightX, rightOpacity)}>
            <div style={tilt(cardTilt, "0% 50%")}>
              <ImageCard
                s={s}
                src={image2}
                kind={image2Kind}
                footage={footageUrls?.image2}
                t={t}
                radius={cardRadius}
              />
            </div>
          </div>
        </Item>
      </AbsoluteFill>

      {edgeFade ? (
        <AbsoluteFill style={{ opacity: vignette, pointerEvents: "none" }}>
          <div style={fade(`linear-gradient(to left, ${t.background} 0%, transparent 30%)`)} />
          <div style={fade(`linear-gradient(to right, ${t.background} 0%, transparent 22%)`)} />
          <div style={fade(`linear-gradient(to top, ${t.background} 0%, transparent 30%)`)} />
        </AbsoluteFill>
      ) : null}

      <div
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          top: 0.22 * height,
          padding: `0 ${40 * s}px`,
          textAlign: "center",
          color: t.foreground,
          fontWeight: headlineWeight,
          letterSpacing: "-0.02em",
          fontSize: headlineSize * s,
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

function ImageCard({
  s,
  src,
  kind,
  footage,
  t,
  radius,
}: {
  s: number;
  src: string;
  kind: MediaKind | undefined;
  footage?: string;
  t: WarmTheme;
  radius: number;
}) {
  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        height: "100%",
        overflow: "hidden",
        borderRadius: radius * s,
        background: mixOklch(t.background, t.card, 0.35),
        boxShadow: `0 ${20 * s}px ${52 * s}px -${28 * s}px ${withAlpha(
          mixOklch(t.background, "#000", 0.75),
          0.9,
        )}, inset 0 0 0 1px ${withAlpha(t.foreground, 0.22)}`,
      }}
    >
      {src ? <CardMedia src={src} kind={kind} footage={footage} /> : null}
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

export function HeroLaunch({
  theme,
  ...settings
}: ElementSettings["hero-launch"] & {
  image1Kind?: MediaKind;
  image2Kind?: MediaKind;
  footageUrls?: FootageUrls;
}) {
  return (
    <WarmUIProvider mode={theme.mode}>
      <HeroLaunchView {...settings} theme={themeColors(theme)} mode={theme.mode} />
    </WarmUIProvider>
  );
}
