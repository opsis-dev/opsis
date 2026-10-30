import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { PathDraw } from "#video/elements/scene/logo-reveal/path-draw.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import { getSafeAreaPadding, scaleFont } from "#video/lib/layout/layout.ts";
import { DELAY, DURATION, EASING } from "#video/lib/motion/motion-tokens.ts";
import { springSmooth } from "#video/lib/motion/springs.ts";

const EXIT_FOR = 0.42;

export function LogoReveal({
  pathD,
  viewBox,
  size,
  wordmark,
  tagline,
  stroke,
  strokeWidth,
  fill,
  backgroundColor,
  textColor,
  holdSeconds,
  fontFamily,
  speed,
  drawSeconds,
  pathStagger,
  nameAt,
  glow,
  penTip,
  nameSize,
  fontWeight,
  taglineSize,
  taglineColor,
}: ElementSettings["logo-reveal"] & { holdSeconds?: number }) {
  const frame = useCurrentFrame() * speed;
  const { width, height, fps } = useVideoConfig();
  const safeArea = getSafeAreaPadding({ width, height });

  const hasCopy = Boolean(wordmark || tagline);
  const markSize = size ?? Math.round(Math.min(width, height) * (hasCopy ? 0.4 : 0.56));
  const lineWidth = strokeWidth ?? Math.max(2, Math.round(markSize * 0.022));
  const drawFrames = drawSeconds * fps;
  const nameFrame = nameAt * fps;
  const taglineFrame = nameFrame + 0.4 * fps;

  const bloom = spring({
    frame,
    fps,
    config: springSmooth,
    durationInFrames: DURATION.slow,
  });

  const wordmarkLift = spring({
    frame: frame - nameFrame,
    fps,
    config: springSmooth,
    durationInFrames: DURATION.normal,
  });
  const taglineIn = interpolate(frame, [taglineFrame, taglineFrame + DURATION.normal], [0, 1], {
    ...CLAMPED,
    easing: EASING.enter,
  });

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [holdSeconds * fps, (holdSeconds + EXIT_FOR) * fps], [0, 1], {
          ...CLAMPED,
          easing: EASING.exit,
        });
  const leaving = {
    opacity: 1 - exit,
    translate: `0 ${exit * -scaleFont(26, width)}px`,
  };

  return (
    <div
      style={{
        width,
        height,
        backgroundColor,
        paddingLeft: safeArea.paddingLeft,
        paddingRight: safeArea.paddingRight,
        paddingTop: safeArea.paddingTop,
        paddingBottom: safeArea.paddingBottom,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: scaleFont(28, width),
        position: "relative",
        fontFamily: fontOr(fontFamily, "main"),
      }}
    >
      <div
        style={{
          position: "relative",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          scale: 0.92 + bloom * 0.08,
          ...leaving,
        }}
      >
        {glow ? (
          <div
            style={{
              position: "absolute",
              width: markSize * 1.8,
              height: markSize * 1.8,
              borderRadius: "50%",
              background: `radial-gradient(circle, color-mix(in srgb, ${stroke} 22%, transparent), transparent 70%)`,
              filter: `blur(${scaleFont(36, width)}px)`,
              opacity: bloom * 0.9,
              scale: 0.8 + bloom * 0.35,
              pointerEvents: "none",
            }}
          />
        ) : null}
        <PathDraw
          d={pathD}
          viewBox={viewBox}
          width={markSize}
          height={markSize}
          stroke={stroke}
          strokeWidth={lineWidth}
          fill={fill}
          delayInFrames={DELAY.short}
          durationInFrames={drawFrames}
          staggerInFrames={pathStagger * fps}
          head={penTip}
          glow={glow}
        />
      </div>

      {wordmark ? (
        <div
          style={{
            color: textColor,
            fontSize: scaleFont(nameSize, width),
            fontWeight,
            lineHeight: 1,
            letterSpacing: -0.5,
            textAlign: "center",
            opacity: wordmarkLift * (1 - exit),
            translate: `0 ${
              (1 - wordmarkLift) * scaleFont(28, width) - exit * scaleFont(26, width)
            }px`,
          }}
        >
          {wordmark}
        </div>
      ) : null}

      {tagline ? (
        <div
          style={{
            color: taglineColor || `color-mix(in srgb, ${textColor} 60%, transparent)`,
            fontSize: scaleFont(taglineSize, width),
            fontWeight: Math.max(100, fontWeight - 100),
            lineHeight: 1.2,
            textAlign: "center",
            maxWidth: width * 0.7,
            opacity: taglineIn * (1 - exit),
            translate: `0 ${
              (1 - taglineIn) * scaleFont(16, width) - exit * scaleFont(26, width)
            }px`,
          }}
        >
          {tagline}
        </div>
      ) : null}
    </div>
  );
}
