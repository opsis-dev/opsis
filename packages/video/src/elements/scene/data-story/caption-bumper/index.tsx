import { interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import type { StoryLook } from "#video/elements/scene/data-story/look.ts";
import { fitHeadline } from "#video/elements/scene/data-story/text-fit-utils.ts";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

export type CaptionBumperProps = {
  text: string;
  eyebrow?: string;
  maxFontSize: number;
  holdSeconds?: number;
  backgroundColor?: string;
  accentColor: string;
  look: StoryLook;
  speed?: number;
};

const T = {
  ground: 0,
  groundFor: 0.4,
  eyebrow: 0.16,
  word: 0.3,
  wordStagger: 0.055,
  wordsBudget: 0.7,
  rule: 0.62,
  exitFor: 0.36,
} as const;

const clamp = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
} as const;

export function CaptionBumper({
  text,
  eyebrow,
  maxFontSize,
  holdSeconds,
  backgroundColor,
  accentColor,
  look,
  speed = 1,
}: CaptionBumperProps) {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const palette = look.palette;
  const safe = getSafeAreaPadding({ width, height });

  const at = (seconds: number) => (seconds * fps) / speed;
  const ease = (from: number, to: number, easing = EASING.enter) =>
    interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...clamp });

  const stage = {
    w: width - safe.paddingLeft - safe.paddingRight,
    h: height - safe.paddingTop - safe.paddingBottom,
  };
  const portrait = height > width;
  const u = portrait
    ? Math.min(stage.w / 620, stage.h / 1120)
    : Math.min(stage.w / 1120, stage.h / 620);

  const words = text.split(/\s+/).filter(Boolean);
  const fontSize = fitHeadline({
    text,
    maxWidth: stage.w,
    maxFontSize: maxFontSize * look.textSize * u,
    minFontSize: 30 * u,
  });

  const ground = ease(T.ground, T.ground + T.groundFor, EASING.editorial);
  const eyebrowIn = eyebrow ? ease(T.eyebrow, T.eyebrow + 0.4) : 0;
  const wordStagger =
    words.length > 1 ? Math.min(T.wordStagger, T.wordsBudget / (words.length - 1)) : 0;
  const lastWord = T.word + (words.length - 1) * wordStagger + 0.4;
  const rule = ease(Math.max(T.rule, lastWord - 0.1), lastWord + 0.4, EASING.editorial);
  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...clamp,
        });

  return (
    <div
      style={{
        width,
        height,
        background: backgroundColor ?? palette.page,
        fontFamily: look.font,
        position: "relative",
        overflow: "hidden",
        display: "grid",
        placeItems: "center",
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `radial-gradient(ellipse 70% 55% at 50% 50%, color-mix(in srgb, ${accentColor} ${13 * look.glow}%, transparent), transparent 72%)`,
          clipPath: `inset(${(1 - ground) * 50}% 0 ${(1 - ground) * 50}% 0)`,
          opacity: 1 - exit,
        }}
      />

      <div
        style={{
          position: "relative",
          width: stage.w,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 16 * u,
          textAlign: "center",
          opacity: 1 - exit,
          translate: `0 ${exit * -22 * u}px`,
        }}
      >
        {eyebrow ? (
          <div
            style={{
              color: accentColor,
              fontSize: 22 * u,
              fontWeight: 600,
              letterSpacing: look.eyebrowCaps ? "0.16em" : undefined,
              textTransform: look.eyebrowCaps ? "uppercase" : "none",
              opacity: eyebrowIn,
              translate: `0 ${(1 - eyebrowIn) * 10 * u}px`,
            }}
          >
            {eyebrow}
          </div>
        ) : null}

        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            justifyContent: "center",
            gap: `${0.1 * fontSize}px ${0.28 * fontSize}px`,
            color: palette.fg,
            fontSize,
            fontWeight: 700,
            lineHeight: 1.06,
            letterSpacing: "-0.03em",
          }}
        >
          {words.map((word, index) => {
            const wordIn = ease(T.word + index * wordStagger, T.word + index * wordStagger + 0.4);
            return (
              <span key={`${word}-${index}`} style={{ display: "block", overflow: "hidden" }}>
                <span
                  style={{
                    display: "block",
                    translate: `0 ${(1 - wordIn) * 100}%`,
                  }}
                >
                  {word}
                </span>
              </span>
            );
          })}
        </div>

        <div
          style={{
            width: fontSize * 2.2,
            height: 4 * u,
            borderRadius: 999,
            background: accentColor,
            boxShadow: `0 0 ${16 * u}px color-mix(in srgb, ${accentColor} 53%, transparent)`,
            transformOrigin: "center",
            scale: `${rule} 1`,
          }}
        />
      </div>
    </div>
  );
}
