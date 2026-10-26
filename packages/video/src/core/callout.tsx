import React from "react";
import { AbsoluteFill, Easing, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED, exitOpacity } from "#video/core/anim.ts";
import { useOverlayFrame } from "#video/core/frame.ts";
import type { CoreElementProps } from "#video/generated/spec.ts";

const CALLOUT = {
  drawFrames: 18,
  offsetPx: 170,
  bendPx: 40,
  strokePx: 7,
  dashSlack: 1.3,
  dotPx: 14,
  labelPaddingPx: [12, 22],
  labelRadiusPx: 14,
  labelSizePx: 34,
  labelWeight: 700,
} as const;

const CALLOUT_DIRECTION = {
  top: [0.35, -1],
  bottom: [0.35, 1],
  left: [-1, -0.35],
  right: [1, -0.35],
} as const;

export const Callout: React.FC<CoreElementProps["core/callout"]> = ({
  text,
  x,
  y,
  side,
  color,
  accent,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const { width, height, durationInFrames } = useVideoConfig();
  const { k } = useOverlayFrame();
  const draw = interpolate(frame, [0, CALLOUT.drawFrames], [0, 1], {
    ...CLAMPED,
    easing: Easing.out(Easing.cubic),
  });
  const off = CALLOUT.offsetPx * k;
  const [dx, dy] = CALLOUT_DIRECTION[side];
  const sx = x * width + dx * off;
  const sy = y * height + dy * off;
  const len = Math.hypot(dx * off, dy * off);
  const [labelPadY, labelPadX] = CALLOUT.labelPaddingPx;
  return (
    <AbsoluteFill style={{ opacity: exitOpacity(frame, durationInFrames) }}>
      <svg width={width} height={height} style={{ position: "absolute", inset: 0 }}>
        <path
          d={`M ${sx} ${sy} Q ${(sx + x * width) / 2 + CALLOUT.bendPx * k} ${(sy + y * height) / 2} ${x * width} ${y * height}`}
          fill="none"
          stroke={accent}
          strokeWidth={CALLOUT.strokePx * k}
          strokeLinecap="round"
          strokeDasharray={len * CALLOUT.dashSlack}
          strokeDashoffset={len * CALLOUT.dashSlack * (1 - draw)}
        />
        <circle cx={x * width} cy={y * height} r={CALLOUT.dotPx * k * draw} fill={accent} />
      </svg>
      <div
        style={{
          position: "absolute",
          left: sx,
          top: sy,
          transform: `translate(-50%, ${dy > 0 ? "0" : "-100%"})`,
          padding: `${labelPadY * k}px ${labelPadX * k}px`,
          borderRadius: CALLOUT.labelRadiusPx * k,
          background: accent,
          color,
          fontFamily: fontOr(fontFamily, "main"),
          fontWeight: CALLOUT.labelWeight,
          fontSize: CALLOUT.labelSizePx * k,
          opacity: draw,
          whiteSpace: "nowrap",
        }}
      >
        {text}
      </div>
    </AbsoluteFill>
  );
};
