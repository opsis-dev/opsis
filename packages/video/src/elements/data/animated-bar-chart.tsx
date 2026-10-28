import { spring } from "remotion";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import { type ElementProps } from "#video/common/spec.ts";
import { listsFrom } from "#video/lib/settings/shape.ts";
import type React from "react";

export interface AnimatedBarChartProps {
  data?: number[];
  labels?: string[];
  width?: number;
  height?: number;
  barColor?: string;
  baselineColor?: string;
  labelColor?: string;
  gap?: number;
  staggerFrames?: number;
  speed?: number;
  className?: string;
}

function AnimatedBarChartView({
  data = [35, 60, 45, 80, 55, 70, 90, 65],
  labels,
  width = 1000,
  height = 500,
  barColor = "#0ea5e9",
  baselineColor = "#27272a",
  labelColor = "#a1a1aa",
  gap = 16,
  staggerFrames = 6,
  speed = 1,
  className,
}: AnimatedBarChartProps) {
  const frame = useAuthoredFrame() * speed;
  const { fps } = useAuthoredConfig();

  const padding = 60;
  const labelSpace = labels ? 40 : 0;
  const innerWidth = width - padding * 2;
  const innerHeight = height - padding * 2 - labelSpace;
  const max = Math.max(...data, 0) || 1;
  const fitGap = Math.max(
    0,
    Math.min(gap, (innerWidth - data.length) / Math.max(1, data.length - 1)),
  );
  const barWidth = Math.max(1, (innerWidth - fitGap * (data.length - 1)) / data.length);
  const baseY = padding + innerHeight;

  return (
    <div
      className={className}
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontFamily:
          "var(--font-geist-sans), -apple-system, BlinkMacSystemFont, sans-serif",
      }}
    >
      <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`}>
        {/* Baseline */}
        <line
          x1={padding}
          x2={padding + innerWidth}
          y1={baseY}
          y2={baseY}
          stroke={baselineColor}
          strokeWidth={2}
        />

        {data.map((value, index) => {
          const targetHeight = (value / max) * innerHeight;
          const x = padding + index * (barWidth + fitGap);
          const y = baseY - targetHeight;

          const scaleY = spring({
            frame: frame - index * staggerFrames,
            fps,
            config: { damping: 12, stiffness: 100, mass: 0.8 },
            from: 0,
            to: 1,
          });

          return (
            <g key={index}>
              <rect
                x={x}
                y={y}
                width={barWidth}
                height={targetHeight}
                rx={6}
                fill={barColor}
                style={{
                  scale: `1 ${scaleY}`,
                  transformOrigin: `${x + barWidth / 2}px ${baseY}px`,
                  transformBox: "fill-box",
                  filter: `drop-shadow(0 4px 16px color-mix(in srgb, ${barColor} 33%, transparent))`,
                }}
              />
              {labels?.[index] && (
                <text
                  x={x + barWidth / 2}
                  y={baseY + 28}
                  fill={labelColor}
                  fontSize={Math.min(16, (barWidth + fitGap * 0.8) / Math.max(1, labels[index].length * 0.56))}
                  textAnchor="middle"
                  style={{
                    opacity: scaleY,
                    fontFamily:
                      "var(--font-geist-sans), -apple-system, BlinkMacSystemFont, sans-serif",
                  }}
                >
                  {labels[index]}
                </text>
              )}
            </g>
          );
        })}
      </svg>
    </div>
  );
}

export function AnimatedBarChart(props: ElementProps & { children?: React.ReactNode }) {
  return <AnimatedBarChartView {...(listsFrom(props, {"bars":{"label":"labels","value":"data"}}) as unknown as React.ComponentProps<typeof AnimatedBarChartView>)} />;
}
