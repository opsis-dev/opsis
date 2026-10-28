import type React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { type ElementProps } from "#video/common/spec.ts";
import { formatCompactNumber, readDelta, type ChartDatum } from "#video/lib/charts/chart-utils.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

export type DonutChartProps = {
  segments: ChartDatum[];
  size?: number;
  thickness?: number;
  colors?: string[];
  showLegend?: boolean;
  showTotal?: boolean;
  totalLabel?: string;
  valueFormatter?: (value: number) => string;
  trackColor?: string;
  labelColor?: string;
  inkColor?: string;
  upColor?: string;
  downColor?: string;
  durationInFrames?: number;
  staggerInFrames?: number;
  delayInFrames?: number;
  exitAtInFrames?: number;
  exitInFrames?: number;
  frame?: number;
};

const DEFAULT_COLORS = ["#e8b86d", "#2dd4bf", "#f472b6", "#8b8bf5"];

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

const DonutChartView: React.FC<DonutChartProps> = ({
  segments,
  size = 360,
  thickness = 46,
  colors = DEFAULT_COLORS,
  showLegend = true,
  showTotal = true,
  totalLabel = "Total",
  valueFormatter = (value: number) => formatCompactNumber(value),
  trackColor = "rgba(250,250,250,0.08)",
  labelColor = "rgba(250,250,250,0.58)",
  inkColor = "#fafafa",
  upColor = "#2dd4bf",
  downColor = "#f87171",
  durationInFrames = 26,
  staggerInFrames = 9,
  delayInFrames = 0,
  exitAtInFrames,
  exitInFrames = 20,
  frame: frameOverride,
}) => {
  const localFrame = useCurrentFrame();
  const frame = frameOverride ?? localFrame;

  const total = segments.reduce((sum, segment) => sum + segment.value, 0) || 1;
  const radius = (size - thickness) / 2;
  const circumference = 2 * Math.PI * radius;
  const gap = Math.min(circumference * 0.012, thickness * 0.22);

  const exit =
    exitAtInFrames === undefined
      ? 0
      : interpolate(frame, [exitAtInFrames, exitAtInFrames + exitInFrames], [0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: EASING.exit,
        });

  const drawn = segments.map((segment, index) => {
    const start = segments.slice(0, index).reduce((sum, before) => sum + before.value / total, 0);
    const fraction = segment.value / total;

    const progress = interpolate(
      frame,
      [
        delayInFrames + index * staggerInFrames,
        delayInFrames + index * staggerInFrames + durationInFrames,
      ],
      [0, 1],
      { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: EASING.enter },
    );

    return {
      ...segment,
      start,
      fraction,
      progress,
      color: segment.color ?? colors[index % colors.length],
    };
  });

  const landed = drawn.reduce((sum, segment) => sum + segment.value * segment.progress, 0);

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: size * 0.16,
        opacity: 1 - clamp01(exit / 0.7),
        scale: `${1 - exit * 0.06}`,
      }}
    >
      <div style={{ position: "relative", width: size, height: size }}>
        <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
          <circle
            cx={size / 2}
            cy={size / 2}
            r={radius}
            fill="none"
            stroke={trackColor}
            strokeWidth={thickness}
          />
          {drawn.map((segment, index) => {
            const length = Math.max(0, segment.fraction * circumference * segment.progress - gap);

            return (
              <circle
                key={index}
                cx={size / 2}
                cy={size / 2}
                r={radius}
                fill="none"
                stroke={segment.color}
                strokeWidth={thickness}
                strokeLinecap="butt"
                strokeDasharray={`${length} ${circumference}`}
                transform={`rotate(${-90 + segment.start * 360} ${size / 2} ${size / 2})`}
              />
            );
          })}
        </svg>

        {showTotal ? (
          <div
            style={{
              position: "absolute",
              inset: 0,
              display: "grid",
              placeItems: "center",
              textAlign: "center",
            }}
          >
            <div>
              <div
                style={{
                  color: inkColor,
                  fontSize: size * 0.17,
                  fontWeight: 700,
                  lineHeight: 1,
                  fontVariantNumeric: "tabular-nums",
                }}
              >
                {valueFormatter(landed)}
              </div>
              <div
                style={{
                  color: labelColor,
                  fontSize: size * 0.065,
                  fontWeight: 600,
                  letterSpacing: "0.06em",
                  textTransform: "uppercase",
                  marginTop: size * 0.03,
                }}
              >
                {totalLabel}
              </div>
            </div>
          </div>
        ) : null}
      </div>

      {showLegend ? (
        <div style={{ display: "grid", gap: size * 0.055 }}>
          {drawn.map((segment, index) => (
            <div
              key={index}
              style={{
                display: "flex",
                alignItems: "center",
                gap: size * 0.045,
                opacity: clamp01(segment.progress * 1.6),
                translate: `${(1 - segment.progress) * size * 0.05}px`,
              }}
            >
              <span
                style={{
                  width: size * 0.05,
                  height: size * 0.05,
                  borderRadius: 999,
                  background: segment.color,
                  flexShrink: 0,
                }}
              />
              <span
                style={{
                  color: inkColor,
                  fontSize: size * 0.062,
                  fontWeight: 600,
                  minWidth: size * 0.42,
                }}
              >
                {segment.label}
              </span>
              <span
                style={{
                  color: labelColor,
                  fontSize: size * 0.062,
                  fontWeight: 600,
                  fontVariantNumeric: "tabular-nums",
                }}
              >
                {Math.round(segment.fraction * segment.progress * 100)}%
              </span>
              {segment.delta ? (
                <span
                  style={{
                    color:
                      readDelta(segment.delta).direction === "down"
                        ? downColor
                        : readDelta(segment.delta).direction === "up"
                          ? upColor
                          : labelColor,
                    fontSize: size * 0.052,
                    fontWeight: 600,
                    fontVariantNumeric: "tabular-nums",
                  }}
                >
                  {readDelta(segment.delta).text}
                </span>
              ) : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
};

export function DonutChart(props: ElementProps & { children?: React.ReactNode }) {
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <DonutChartView {...(props as unknown as React.ComponentProps<typeof DonutChartView>)} />
    </AbsoluteFill>
  );
}
