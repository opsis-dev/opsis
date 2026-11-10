import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import React from "react";
import { fontOr } from "#video/common/fonts.ts";
import { entranceStyle, exitOpacity, type Entrance } from "#video/core/anim.ts";
import { GradientScaleCutText } from "#video/elements/text/gradient-scale-cut-text.tsx";
import { HandwritingText } from "#video/elements/text/handwriting-text.tsx";
import { KineticHeadline } from "#video/elements/text/kinetic-headline.tsx";
import { LineByLineSlide } from "#video/elements/text/line-by-line-slide.tsx";
import { MaskRevealUp } from "#video/elements/text/mask-reveal-up.tsx";
import { MatrixDecode } from "#video/elements/text/matrix-decode.tsx";
import { NeonFlickerText } from "#video/elements/text/neon-flicker-text.tsx";
import { RGBGlitchText } from "#video/elements/text/rgb-glitch-text.tsx";
import { SoftBlurIn } from "#video/elements/text/soft-blur-in.tsx";
import { StaggeredFadeUp } from "#video/elements/text/staggered-fade-up.tsx";
import { TrackingIn } from "#video/elements/text/tracking-in.tsx";
import { Typewriter } from "#video/elements/text/typewriter.tsx";
import { WordPush } from "#video/elements/text/word-push.tsx";
import type { ElementSettings, Entrance as TitleEntrance } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";
import DEFAULT_PROPS from "#video/registry/default-props.generated.json";

const SUBTITLE_FADE = 0.7;

type LineLook = { sizeEm: number; delay: number; style: React.CSSProperties };

const LINES = {
  eyebrow: {
    sizeEm: 0.26,
    delay: 0,
    style: { fontWeight: 700, letterSpacing: "0.18em" },
  },
  title: {
    sizeEm: 1,
    delay: 4,
    style: { fontWeight: 760, lineHeight: 1.02, maxWidth: "100%" },
  },
  subtitle: {
    sizeEm: 0.34,
    delay: 10,
    style: { fontWeight: 450, maxWidth: "80%" },
  },
} satisfies Record<string, LineLook>;

type AnimatedText = { key: keyof typeof DEFAULT_PROPS; draw: (props: never) => React.ReactNode };

const ANIMATED: Partial<Record<TitleEntrance, AnimatedText>> = {
  "soft-blur": { key: "soft-blur-in", draw: SoftBlurIn },
  typewriter: { key: "typewriter", draw: Typewriter },
  "fade-up": { key: "staggered-fade-up", draw: StaggeredFadeUp },
  "word-push": { key: "word-push", draw: WordPush },
  "mask-reveal": { key: "mask-reveal-up", draw: MaskRevealUp },
  tracking: { key: "tracking-in", draw: TrackingIn },
  glitch: { key: "rgb-glitch-text", draw: RGBGlitchText },
  decode: { key: "matrix-decode", draw: MatrixDecode },
  "line-by-line": { key: "line-by-line-slide", draw: LineByLineSlide },
  headline: { key: "kinetic-headline", draw: KineticHeadline },
  handwriting: { key: "handwriting-text", draw: HandwritingText },
  neon: { key: "neon-flicker-text", draw: NeonFlickerText },
  "scale-cut": { key: "gradient-scale-cut-text", draw: GradientScaleCutText },
};

const LINE_HEIGHT = 1.15;

function weightFor(usual: unknown, weight: number): string | number {
  if (typeof usual !== "string") return weight;
  return String(Math.min(700, Math.max(400, Math.round(weight / 100) * 100)));
}

type AnimatedLineProps = {
  animated: AnimatedText;
  text: string;
  color: string;
  fontSize: number;
  fontWeight: number;
  fontFamily: string;
  letterSpacing: number;
};

function AnimatedLine({
  animated,
  text,
  color,
  fontSize,
  fontWeight,
  fontFamily,
  letterSpacing,
}: AnimatedLineProps) {
  const usual: Record<string, unknown> = DEFAULT_PROPS[animated.key];
  const lines = Math.max(1, text.split("\n").length);
  const Draw = animated.draw as (props: Record<string, unknown>) => React.ReactNode;
  return (
    <div style={{ position: "relative", width: "100%", height: fontSize * LINE_HEIGHT * lines }}>
      <Draw
        {...usual}
        text={text}
        color={color}
        fontSize={fontSize}
        fontWeight={weightFor(usual.fontWeight, fontWeight)}
        fontFamily={fontFamily}
        letterSpacing={letterSpacing}
        background="transparent"
      />
    </div>
  );
}

type TitleLineProps = {
  look: LineLook;
  text: React.ReactNode;
  color: string;
  fontSize: number;
  entrance: TitleEntrance;
  stagger: number;
};

function lineEntrance(entrance: TitleEntrance): Entrance {
  return ANIMATED[entrance] ? "fade" : (entrance as Entrance);
}

function TitleLine({ look, text, color, fontSize, entrance, stagger }: TitleLineProps) {
  const frame = useFrame();
  const { fps } = useFilm();
  return (
    <div
      style={{
        ...entranceStyle(lineEntrance(entrance), frame, fps, look.delay * stagger),
        color,
        fontSize: fontSize * look.sizeEm,
        ...look.style,
      }}
    >
      {text}
    </div>
  );
}

export function Title({
  eyebrow,
  title,
  subtitle,
  align,
  entrance,
  color,
  accent,
  background,
  fontSize,
  fontWeight,
  fontFamily,
  sideMargin,
  lineGap,
  eyebrowSize,
  eyebrowUppercase,
  subtitleSize,
  letterSpacing,
  lineStagger,
  fadeOut,
  subtitleColor,
}: ElementSettings["title"]) {
  const frame = useFrame();
  const { durationInFrames, fps } = useFilm();
  const out = exitOpacity(frame, durationInFrames, Math.round(fadeOut * fps));
  const line = { fontSize, entrance, stagger: lineStagger };
  const animated = ANIMATED[entrance];
  return (
    <AbsoluteFill>
      <Backgrounded background={background} />
      <AbsoluteFill
        style={{
          alignItems: align === "center" ? "center" : "flex-start",
          justifyContent: "center",
          padding: `0 ${sideMargin}%`,
          textAlign: align,
          fontFamily: fontOr(fontFamily, "heading"),
          opacity: out,
          gap: fontSize * lineGap,
        }}
      >
        {eyebrow ? (
          <TitleLine
            look={{
              ...LINES.eyebrow,
              sizeEm: eyebrowSize,
              style: {
                ...LINES.eyebrow.style,
                textTransform: eyebrowUppercase ? "uppercase" : "none",
              },
            }}
            text={eyebrow}
            color={accent}
            {...line}
          />
        ) : null}
        {animated ? (
          <AnimatedLine
            animated={animated}
            text={title}
            color={color}
            fontSize={fontSize}
            fontWeight={fontWeight}
            fontFamily={fontOr(fontFamily, "heading")}
            letterSpacing={letterSpacing}
          />
        ) : (
          <TitleLine
            look={{
              ...LINES.title,
              style: { ...LINES.title.style, fontWeight, letterSpacing: `${letterSpacing}em` },
            }}
            text={title}
            color={color}
            {...line}
          />
        )}
        {subtitle ? (
          <TitleLine
            look={{
              ...LINES.subtitle,
              sizeEm: subtitleSize,
              style: { ...LINES.subtitle.style, opacity: subtitleColor ? 1 : SUBTITLE_FADE },
            }}
            text={subtitle}
            color={subtitleColor || color}
            {...line}
          />
        ) : null}
      </AbsoluteFill>
    </AbsoluteFill>
  );
}
