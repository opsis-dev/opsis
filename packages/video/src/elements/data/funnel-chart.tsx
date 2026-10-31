import { useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { fontOr } from "#video/common/fonts.ts";
import { clamp01 } from "#video/common/math.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { formatNumber } from "#video/lib/charts/chart-utils.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const EXIT_FRAMES = 20;

export function FunnelChart({
  stages,
  width,
  height,
  gap,
  color,
  tailOpacity,
  inkColor,
  labelColor,
  labelWidth,
  showDropoff,
  dropoffColor,
  showConversion,
  durationInFrames,
  staggerInFrames,
  numberFormat,
  dropoffHeading,
  fontFamily,
  exitAtInFrames,
}: ElementSettings["funnel-chart"] & { exitAtInFrames?: number }) {
  const frame = useFrame();
  const delayInFrames = 0;
  const exitInFrames = EXIT_FRAMES;

  const count = Math.max(1, stages.length);
  const peak = stages.reduce((highest, stage) => Math.max(highest, stage.value), 0) || 1;
  const bandHeight = (height - gap * (count - 1)) / count;
  const labelSize = Math.max(12, Math.min(bandHeight * 0.34, width * 0.028));

  const gutter = labelWidth ?? width * 0.3;
  const dropoffGutter = showDropoff ? width * 0.13 : 0;
  const funnelWidth = width - gutter - dropoffGutter;
  const centre = gutter + funnelWidth / 2;

  const exit =
    exitAtInFrames === undefined
      ? 0
      : interpolate(frame, [exitAtInFrames, exitAtInFrames + exitInFrames], [0, 1], {
          ...CLAMPED,
          easing: EASING.exit,
        });

  function bandWidth(value: number) {
    return (Math.max(0, value) / (peak || 1)) * funnelWidth;
  }

  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <svg
        width={width}
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        style={{ fontFamily: fontOr(fontFamily, "main") }}
      >
        {stages.map((stage, index) => {
          const top = index * (bandHeight + gap);
          const bottom = top + bandHeight;
          const topWidth = bandWidth(stage.value);
          const next = stages[index + 1];
          const bottomWidth = next ? bandWidth(next.value) : topWidth;

          const start = delayInFrames + index * staggerInFrames;
          const progress = interpolate(frame, [start, start + durationInFrames], [0, 1], {
            ...CLAMPED,
            easing: EASING.enter,
          });
          const drain = clamp01((exit - ((count - 1 - index) / count) * 0.3) / 0.7);
          const shown = progress * (1 - drain);
          if (shown <= 0) return null;

          const revealBottom = top + bandHeight * shown;
          const revealWidth = topWidth + (bottomWidth - topWidth) * shown;
          const path = [
            `M ${centre - topWidth / 2} ${top}`,
            `L ${centre + topWidth / 2} ${top}`,
            `L ${centre + revealWidth / 2} ${revealBottom}`,
            `L ${centre - revealWidth / 2} ${revealBottom}`,
            "Z",
          ].join(" ");

          const conversion = Math.round((stage.value / (stages[0]?.value || 1)) * 100);
          const dropoff = next ? Math.round(((next.value - stage.value) / stage.value) * 100) : 0;
          const dropoffProgress = interpolate(
            frame,
            [
              start + staggerInFrames + durationInFrames * 0.6,
              start + staggerInFrames + durationInFrames * 0.6 + 12,
            ],
            [0, 1],
            { ...CLAMPED, easing: EASING.enter },
          );

          return (
            <g key={index}>
              <path
                d={path}
                fill={stage.color ?? color}
                opacity={
                  (tailOpacity + (1 - tailOpacity) * (1 - index / Math.max(1, count - 1))) *
                  (1 - drain)
                }
              />

              <text
                x={gutter - labelSize}
                y={top + bandHeight * 0.36}
                fill={inkColor}
                fontSize={labelSize}
                fontWeight={600}
                textAnchor="end"
                dominantBaseline="central"
                opacity={clamp01((shown - 0.25) / 0.35)}
              >
                {stage.label}
              </text>
              <text
                x={gutter - labelSize}
                y={top + bandHeight * 0.72}
                fill={labelColor}
                fontSize={labelSize * 0.92}
                fontWeight={600}
                textAnchor="end"
                dominantBaseline="central"
                opacity={clamp01((shown - 0.45) / 0.35)}
                style={{ fontVariantNumeric: "tabular-nums" }}
              >
                {formatNumber(stage.value, numberFormat)}
                {showConversion ? ` · ${conversion}%` : ""}
              </text>

              {showDropoff && next && stage.value > 0 ? (
                <text
                  x={width - labelSize * 0.5}
                  y={bottom + gap / 2}
                  fill={dropoffColor}
                  fontSize={labelSize * 0.92}
                  fontWeight={700}
                  textAnchor="end"
                  dominantBaseline="central"
                  opacity={dropoffProgress * (1 - clamp01(exit / 0.6))}
                  style={{ fontVariantNumeric: "tabular-nums" }}
                >
                  {dropoff >= 0 ? "+" : "−"}
                  {Math.abs(dropoff)}%
                </text>
              ) : null}

              {showDropoff && index === 0 ? (
                <text
                  x={width - labelSize * 0.5}
                  y={top + bandHeight * 0.32}
                  fill={labelColor}
                  fontSize={labelSize * 0.82}
                  fontWeight={600}
                  textAnchor="end"
                  dominantBaseline="central"
                  opacity={clamp01((shown - 0.4) / 0.3)}
                >
                  {dropoffHeading}
                </text>
              ) : null}
            </g>
          );
        })}
      </svg>
    </AbsoluteFill>
  );
}
