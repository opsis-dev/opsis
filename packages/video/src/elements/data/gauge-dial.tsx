import type React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { type ElementProps } from "#video/common/spec.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

export type GaugeDialProps = {
  value: number;
  min?: number;
  max?: number;
  size?: number;
  thickness?: number;
  sweepInDegrees?: number;
  color?: string;
  trackColor?: string;
  inkColor?: string;
  labelColor?: string;
  label?: string;
  unit?: string;
  tickCount?: number;
  valueFormatter?: (value: number) => string;
  durationInFrames?: number;
  delayInFrames?: number;
  exitAtInFrames?: number;
  exitInFrames?: number;
  frame?: number;
};

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

function polar(cx: number, cy: number, radius: number, degrees: number) {
  const radians = ((degrees - 90) * Math.PI) / 180;
  return { x: cx + radius * Math.cos(radians), y: cy + radius * Math.sin(radians) };
}

function arcPath(
  cx: number,
  cy: number,
  radius: number,
  startAngle: number,
  endAngle: number,
): string {
  if (endAngle - startAngle <= 0.001) return "";
  const start = polar(cx, cy, radius, startAngle);
  const middle = polar(cx, cy, radius, (startAngle + endAngle) / 2);
  const end = polar(cx, cy, radius, endAngle);
  return `M ${start.x} ${start.y} A ${radius} ${radius} 0 0 1 ${middle.x} ${middle.y} A ${radius} ${radius} 0 0 1 ${end.x} ${end.y}`;
}

function decimalsOf(value: number): number {
  return Math.min(3, (String(value).split(".")[1] ?? "").length);
}

const GaugeDialView: React.FC<GaugeDialProps> = ({
  value,
  min = 0,
  max = 100,
  size = 360,
  thickness = 26,
  sweepInDegrees: sweepSetting = 250,
  color = "#e8b86d",
  trackColor = "rgba(250,250,250,0.10)",
  inkColor = "#fafafa",
  labelColor = "rgba(250,250,250,0.58)",
  label,
  unit = "",
  tickCount = 9,
  valueFormatter,
  durationInFrames = 44,
  delayInFrames = 0,
  exitAtInFrames,
  exitInFrames = 20,
  frame: frameOverride,
}) => {
  const localFrame = useCurrentFrame();
  const frame = frameOverride ?? localFrame;

  const sweepInDegrees = Math.min(360, Math.max(1, sweepSetting));
  const fullCircle = sweepInDegrees >= 360;
  const tickSteps = fullCircle ? tickCount : tickCount - 1;
  const format = valueFormatter ?? ((input: number) => input.toFixed(decimalsOf(value)));
  const centre = size / 2;
  const radius = (size - thickness) / 2;
  const startAngle = -sweepInDegrees / 2;
  const endAngle = sweepInDegrees / 2;

  const span = max - min || 1;
  const target = clamp01((value - min) / span);

  const enterWindow: [number, number] = [delayInFrames, delayInFrames + durationInFrames];
  const arcProgress = interpolate(frame, enterWindow, [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: EASING.enter,
  });
  const needleProgress = interpolate(frame, enterWindow, [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: EASING.pop,
  });

  const exit =
    exitAtInFrames === undefined
      ? 0
      : interpolate(frame, [exitAtInFrames, exitAtInFrames + exitInFrames], [0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: EASING.exit,
        });

  const arcFraction = target * arcProgress * (1 - exit);
  const needleFraction = target * needleProgress * (1 - exit);
  const needleAngle = startAngle + needleFraction * sweepInDegrees;
  const needleTip = polar(centre, centre, radius - thickness * 0.55, needleAngle);
  const needleTail = polar(centre, centre, thickness * 0.5, needleAngle + 180);

  return (
    <div
      style={{
        position: "relative",
        width: size,
        height: size,
        opacity: 1 - clamp01(exit / 0.85),
      }}
    >
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        <path
          d={arcPath(centre, centre, radius, startAngle, endAngle)}
          fill="none"
          stroke={trackColor}
          strokeWidth={thickness}
          strokeLinecap="round"
        />
        <path
          d={arcPath(centre, centre, radius, startAngle, startAngle + arcFraction * sweepInDegrees)}
          fill="none"
          stroke={color}
          strokeWidth={thickness}
          strokeLinecap="round"
        />

        {tickCount > 0
          ? Array.from({ length: tickCount }, (_, index) => {
              const fraction = tickSteps > 0 ? index / tickSteps : 0;
              const angle = startAngle + fraction * sweepInDegrees;
              const outer = polar(centre, centre, radius - thickness * 0.72, angle);
              const inner = polar(centre, centre, radius - thickness * 1.05, angle);
              const passed = fraction <= needleFraction;

              return (
                <line
                  key={angle}
                  x1={inner.x}
                  y1={inner.y}
                  x2={outer.x}
                  y2={outer.y}
                  stroke={passed ? inkColor : labelColor}
                  strokeWidth={size * 0.007}
                  strokeLinecap="round"
                  opacity={passed ? 0.85 : 0.35}
                />
              );
            })
          : null}

        <line
          x1={needleTail.x}
          y1={needleTail.y}
          x2={needleTip.x}
          y2={needleTip.y}
          stroke={inkColor}
          strokeWidth={size * 0.018}
          strokeLinecap="round"
        />
        <circle cx={centre} cy={centre} r={size * 0.038} fill={inkColor} />
        <circle cx={centre} cy={centre} r={size * 0.016} fill={color} />
      </svg>

      <div
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          top: size * 0.62,
          textAlign: "center",
        }}
      >
        <div
          style={{
            color: inkColor,
            fontSize: size * 0.14,
            fontWeight: 700,
            lineHeight: 1,
            fontVariantNumeric: "tabular-nums",
          }}
        >
          {format(min + (value - min) * arcProgress * (1 - exit))}
          {unit}
        </div>
        {label ? (
          <div
            style={{
              color: labelColor,
              fontSize: size * 0.055,
              fontWeight: 600,
              letterSpacing: "0.06em",
              textTransform: "uppercase",
              marginTop: size * 0.028,
            }}
          >
            {label}
          </div>
        ) : null}
      </div>
    </div>
  );
};

export function GaugeDial(props: ElementProps & { children?: React.ReactNode }) {
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <GaugeDialView {...(props as unknown as React.ComponentProps<typeof GaugeDialView>)} />
    </AbsoluteFill>
  );
}
