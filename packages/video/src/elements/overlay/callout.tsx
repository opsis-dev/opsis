import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED, exitOpacity } from "#video/core/anim.ts";
import { useOverlayFrame } from "#video/core/frame.ts";
import type { ElementSettings, Side } from "#video/generated/spec.ts";

const CALLOUT = {
  dashSlack: 1.3,
  labelPaddingPx: [12, 22],
} as const;

function directionOf(side: Side, slant: number): [number, number] {
  const s = Math.tan((slant * Math.PI) / 180);
  switch (side) {
    case "top":
      return [s, -1];
    case "bottom":
      return [s, 1];
    case "left":
      return [-1, -s];
    case "right":
      return [1, -s];
  }
}

export function Callout({
  text,
  x,
  y,
  side,
  color,
  accent,
  fontFamily,
  drawTime,
  labelDistance,
  bend,
  arrowWidth,
  dotSize,
  labelRadius,
  fontSize,
  fontWeight,
  slant,
  fadeOut,
  labelBackground,
}: ElementSettings["callout"]) {
  const frame = useFrame();
  const { width, height, durationInFrames, fps } = useFilm();
  const { k } = useOverlayFrame();
  const draw = interpolate(frame, [0, drawTime * fps], [0, 1], {
    ...CLAMPED,
    easing: Easing.out(Easing.cubic),
  });
  const off = labelDistance * k;
  const [dx, dy] = directionOf(side, slant);
  const sx = x * width + dx * off;
  const sy = y * height + dy * off;
  const len = Math.hypot(dx * off, dy * off);
  const [labelPadY, labelPadX] = CALLOUT.labelPaddingPx;
  return (
    <AbsoluteFill
      style={{ opacity: exitOpacity(frame, durationInFrames, Math.round(fadeOut * fps)) }}
    >
      <svg width={width} height={height} style={{ position: "absolute", inset: 0 }}>
        <path
          d={`M ${sx} ${sy} Q ${(sx + x * width) / 2 + bend * k} ${(sy + y * height) / 2} ${x * width} ${y * height}`}
          fill="none"
          stroke={accent}
          strokeWidth={arrowWidth * k}
          strokeLinecap="round"
          strokeDasharray={len * CALLOUT.dashSlack}
          strokeDashoffset={len * CALLOUT.dashSlack * (1 - draw)}
        />
        <circle cx={x * width} cy={y * height} r={dotSize * k * draw} fill={accent} />
      </svg>
      <div
        style={{
          position: "absolute",
          left: sx,
          top: sy,
          transform: `translate(-50%, ${dy > 0 ? "0" : "-100%"})`,
          padding: `${labelPadY * k}px ${labelPadX * k}px`,
          borderRadius: labelRadius * k,
          background: labelBackground || accent,
          color,
          fontFamily: fontOr(fontFamily, "main"),
          fontWeight,
          fontSize: fontSize * k,
          opacity: draw,
          whiteSpace: "nowrap",
        }}
      >
        {text}
      </div>
    </AbsoluteFill>
  );
}
