import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { spring } from "@opsis/runtime/spring.ts";
import { fontOr } from "#video/common/fonts.ts";
import { isLandscape, overlayPadding, useOverlayFrame } from "#video/core/frame.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

const LOWER_THIRD_SPRING = { damping: 200 };

const LOWER_THIRD = {
  gapPx: 18,
  panelPaddingPx: [18, 28],
  titleLetterSpacing: "-0.02em",
  subtitleGapPx: 4,
} as const;

function slideOffset(enter: number, exit: number, dir: number, slide: number): number {
  return slide * dir * (1 - enter + exit);
}

export function LowerThird({
  title,
  subtitle,
  position,
  accent,
  color,
  panel,
  fontFamily,
  exitTime,
  slide,
  margin,
  barWidth,
  cornerRadius,
  panelBlur,
  titleSize,
  titleWeight,
  subtitleSize,
  subtitleOpacity,
}: ElementSettings["lower-third"]) {
  const frame = useFrame();
  const { fps, durationInFrames, width, height } = useFilm();
  const { k, insets } = useOverlayFrame();
  const enter = spring({ frame, fps, config: LOWER_THIRD_SPRING });
  const exit = spring({
    frame: frame - (durationInFrames - Math.round(exitTime * fps)),
    fps,
    config: LOWER_THIRD_SPRING,
  });
  const top = position.startsWith("top") || !isLandscape(width, height);
  const right = position.endsWith("right");
  const x = slideOffset(enter, exit, right ? 1 : -1, slide);
  const [panelPadY, panelPadX] = LOWER_THIRD.panelPaddingPx;
  return (
    <AbsoluteFill
      style={{
        justifyContent: top ? "flex-start" : "flex-end",
        alignItems: right ? "flex-end" : "flex-start",
        ...overlayPadding(margin * k, insets),
      }}
    >
      <div
        style={{
          display: "flex",
          gap: LOWER_THIRD.gapPx * k,
          alignItems: "stretch",
          transform: `translateX(${x * k}px)`,
          opacity: Math.min(enter, 1 - exit),
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        <div
          style={{
            width: barWidth * k,
            borderRadius: 99,
            background: accent,
            transform: `scaleY(${enter})`,
            transformOrigin: "50% 100%",
          }}
        />
        <div
          style={{
            background: panel,
            borderRadius: cornerRadius * k,
            padding: `${panelPadY * k}px ${panelPadX * k}px`,
            backdropFilter: `blur(${panelBlur}px)`,
          }}
        >
          <div
            style={{
              color,
              fontSize: titleSize * k,
              fontWeight: titleWeight,
              letterSpacing: LOWER_THIRD.titleLetterSpacing,
            }}
          >
            {title}
          </div>
          {subtitle ? (
            <div
              style={{
                color,
                opacity: subtitleOpacity,
                fontSize: subtitleSize * k,
                marginTop: LOWER_THIRD.subtitleGapPx * k,
              }}
            >
              {subtitle}
            </div>
          ) : null}
        </div>
      </div>
    </AbsoluteFill>
  );
}
