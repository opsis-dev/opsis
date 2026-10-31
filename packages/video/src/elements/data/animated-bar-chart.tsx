import { spring } from "@opsis/runtime/spring.ts";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function AnimatedBarChart({
  bars,
  width,
  height,
  barColor,
  baselineColor,
  labelColor,
  gap,
  staggerFrames,
  speed,
  cornerRadius,
  glow,
  labelSize,
  fontFamily,
}: ElementSettings["animated-bar-chart"]) {
  const data = bars.map((bar) => bar.value);
  const labels = bars.some((bar) => bar.label) ? bars.map((bar) => bar.label) : undefined;
  const face = fontOr(fontFamily, "main");
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
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: face,
      }}
    >
      <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`}>
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
                rx={cornerRadius}
                fill={barColor}
                style={{
                  scale: `1 ${scaleY}`,
                  transformOrigin: `${x + barWidth / 2}px ${baseY}px`,
                  transformBox: "fill-box",
                  filter: `drop-shadow(0 4px 16px color-mix(in srgb, ${barColor} ${glow}%, transparent))`,
                }}
              />
              {labels?.[index] && (
                <text
                  x={x + barWidth / 2}
                  y={baseY + 28}
                  fill={labelColor}
                  fontSize={Math.min(
                    labelSize,
                    (barWidth + fitGap * 0.8) / Math.max(1, labels[index].length * 0.56),
                  )}
                  textAnchor="middle"
                  style={{
                    opacity: scaleY,
                    fontFamily: face,
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
