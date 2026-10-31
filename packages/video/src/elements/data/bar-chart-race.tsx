import { useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { CSSProperties } from "react";
import { fontOr } from "#video/common/fonts.ts";
import { clamp01 } from "#video/common/math.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { Corner, ElementSettings } from "#video/generated/spec.ts";
import { formatNumber } from "#video/lib/charts/chart-utils.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const EXIT_FRAMES = 20;

const CORNER_STYLE: Record<Corner, CSSProperties> = {
  "top-left": { left: 0, top: 0 },
  "top-right": { right: 0, top: 0 },
  "bottom-left": { left: 0, bottom: 0 },
  "bottom-right": { right: 0, bottom: 0 },
};

function valueAt(values: number[], time: number): number {
  if (values.length === 0) return 0;
  const clamped = Math.min(values.length - 1, Math.max(0, time));
  const lower = Math.floor(clamped);
  const upper = Math.min(values.length - 1, lower + 1);
  const t = clamped - lower;
  return values[lower] + (values[upper] - values[lower]) * t;
}

export function BarChartRace({
  series,
  steps,
  framesPerStep,
  visibleRows,
  width,
  rowHeight,
  gap,
  colors,
  inkColor,
  barInkColor,
  labelColor,
  labelWidth,
  showStepLabel,
  numberFormat,
  cornerRadius,
  stepLabelPosition,
  stepLabelSize,
  stepLabelOpacity,
  fontFamily,
  exitAtInFrames,
}: ElementSettings["bar-chart-race"] & { exitAtInFrames?: number }) {
  const frame = useFrame();
  const delayInFrames = 0;
  const exitInFrames = EXIT_FRAMES;

  const stepCount = series.reduce((longest, entry) => Math.max(longest, entry.values.length), 0);
  const lastStep = Math.max(0, stepCount - 1);
  const time = Math.min(
    lastStep,
    Math.max(0, (frame - delayInFrames) / Math.max(1, framesPerStep)),
  );

  const current = series.map((entry, index) => ({
    ...entry,
    current: valueAt(entry.values, time),
    present: clamp01(entry.values.length - time),
    color: entry.color ?? colors[index % colors.length],
  }));

  const leader =
    current.reduce(
      (highest, entry) => (entry.present > 0 ? Math.max(highest, entry.current) : highest),
      0,
    ) || 1;

  function ranksAt(step: number) {
    const live = series
      .map((entry, index) => ({
        index,
        value: entry.values[step],
        live: step < entry.values.length,
      }))
      .filter((entry) => entry.live)
      .sort((a, b) => b.value - a.value || a.index - b.index);
    return series.map((_, index) => {
      const place = live.findIndex((entry) => entry.index === index);
      return place === -1 ? live.length : place;
    });
  }
  const lowerStep = Math.floor(time);
  const lowerRanks = ranksAt(lowerStep);
  const upperRanks = ranksAt(Math.min(lastStep, lowerStep + 1));
  const blend = time - lowerStep;
  const eased = blend * blend * (3 - 2 * blend);

  const exit =
    exitAtInFrames === undefined
      ? 0
      : interpolate(frame, [exitAtInFrames, exitAtInFrames + exitInFrames], [0, 1], {
          ...CLAMPED,
          easing: EASING.exit,
        });

  const pitch = rowHeight + gap;
  const barWidth = width - labelWidth;
  const stepLabel = steps?.[Math.round(time)] ?? "";

  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <div
        style={{
          position: "relative",
          width,
          height: pitch * visibleRows,
          opacity: 1 - clamp01(exit / 0.8),
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        {current.map((entry, index) => {
          const rank = lowerRanks[index] + (upperRanks[index] - lowerRanks[index]) * eased;

          const visibility = clamp01(visibleRows - rank) * entry.present;
          if (visibility <= 0) return null;

          return (
            <div
              key={index}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width,
                height: rowHeight,
                display: "flex",
                alignItems: "center",
                gap: gap,
                translate: `0 ${rank * pitch}px`,
                opacity: visibility,
              }}
            >
              <span
                style={{
                  width: labelWidth - gap,
                  textAlign: "right",
                  color: labelColor,
                  fontSize: rowHeight * 0.32,
                  fontWeight: 600,
                  whiteSpace: "nowrap",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                }}
              >
                {entry.label}
              </span>

              <div
                style={{
                  height: rowHeight,
                  width: Math.max(
                    rowHeight * 0.6,
                    (entry.current / leader) * barWidth * (1 - exit),
                  ),
                  borderRadius: (rowHeight * cornerRadius) / 100,
                  background: entry.color,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "flex-end",
                  paddingRight: rowHeight * 0.28,
                  boxSizing: "border-box",
                }}
              >
                <span
                  style={{
                    color: barInkColor,
                    fontSize: rowHeight * 0.3,
                    fontWeight: 700,
                    fontVariantNumeric: "tabular-nums",
                  }}
                >
                  {formatNumber(entry.current, numberFormat)}
                </span>
              </div>
            </div>
          );
        })}

        {showStepLabel && stepLabel !== "" ? (
          <div
            style={{
              position: "absolute",
              ...CORNER_STYLE[stepLabelPosition],
              color: inkColor,
              fontSize: (rowHeight * stepLabelSize) / 100,
              fontWeight: 700,
              opacity: stepLabelOpacity / 100,
              fontVariantNumeric: "tabular-nums",
            }}
          >
            {stepLabel}
          </div>
        ) : null}
      </div>
    </AbsoluteFill>
  );
}
