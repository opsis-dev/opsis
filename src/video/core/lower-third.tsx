import React from "react";
import { AbsoluteFill, spring, useCurrentFrame, useVideoConfig } from "remotion";
import type { CoreElementProps } from "../../shared/elements/core";
import { fontOr } from "../../shared/fonts";
import { isLandscape, overlayPadding, useOverlayFrame } from "./frame";

const LOWER_THIRD_SPRING = { damping: 200 };
const LOWER_THIRD_EXIT_FRAMES = 14;
const LOWER_THIRD_SLIDE_PX = 120;

const LOWER_THIRD = {
  paddingPx: 80,
  gapPx: 18,
  barWidthPx: 8,
  panelRadiusPx: 18,
  panelPaddingPx: [18, 28],
  panelBlur: "blur(12px)",
  titleSizePx: 44,
  titleWeight: 720,
  titleLetterSpacing: "-0.02em",
  subtitleSizePx: 28,
  subtitleOpacity: 0.7,
  subtitleGapPx: 4,
} as const;

const slideOffset = (enter: number, exit: number, dir: number): number =>
  (1 - enter) * LOWER_THIRD_SLIDE_PX * dir + exit * LOWER_THIRD_SLIDE_PX * dir;

export const LowerThird: React.FC<CoreElementProps["core/lower-third"]> = ({
  title,
  subtitle,
  position,
  accent,
  color,
  panel,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames, width, height } = useVideoConfig();
  const { k, insets } = useOverlayFrame();
  const enter = spring({ frame, fps, config: LOWER_THIRD_SPRING });
  const exit = spring({
    frame: frame - (durationInFrames - LOWER_THIRD_EXIT_FRAMES),
    fps,
    config: LOWER_THIRD_SPRING,
  });
  const top = position.startsWith("top") || !isLandscape(width, height);
  const right = position.endsWith("right");
  const x = slideOffset(enter, exit, right ? 1 : -1);
  const [panelPadY, panelPadX] = LOWER_THIRD.panelPaddingPx;
  return (
    <AbsoluteFill
      style={{
        justifyContent: top ? "flex-start" : "flex-end",
        alignItems: right ? "flex-end" : "flex-start",
        ...overlayPadding(LOWER_THIRD.paddingPx * k, insets),
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
            width: LOWER_THIRD.barWidthPx * k,
            borderRadius: 99,
            background: accent,
            transform: `scaleY(${enter})`,
            transformOrigin: "50% 100%",
          }}
        />
        <div
          style={{
            background: panel,
            borderRadius: LOWER_THIRD.panelRadiusPx * k,
            padding: `${panelPadY * k}px ${panelPadX * k}px`,
            backdropFilter: LOWER_THIRD.panelBlur,
          }}
        >
          <div
            style={{
              color,
              fontSize: LOWER_THIRD.titleSizePx * k,
              fontWeight: LOWER_THIRD.titleWeight,
              letterSpacing: LOWER_THIRD.titleLetterSpacing,
            }}
          >
            {title}
          </div>
          {subtitle ? (
            <div
              style={{
                color,
                opacity: LOWER_THIRD.subtitleOpacity,
                fontSize: LOWER_THIRD.subtitleSizePx * k,
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
};
