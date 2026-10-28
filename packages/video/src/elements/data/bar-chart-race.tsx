import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { formatCompactNumber } from "#video/lib/charts/chart-utils.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";
import { type ElementProps } from "#video/common/spec.ts";
import type React from "react";

export type RaceSeries = {
  label: string;
  /** One value per keyframe. A series with fewer values leaves the board after its last one. */
  values: number[];
  color?: string;
};

export type BarChartRaceProps = {
  series: RaceSeries[];
  /** Keyframe labels, e.g. years. Shown as the running caption. */
  steps?: string[];
  /** Frames spent travelling between two keyframes. */
  framesPerStep?: number;
  /** How many rows are visible. Series ranked below this fade out. */
  visibleRows?: number;
  width?: number;
  rowHeight?: number;
  /** Space between rows. */
  gap?: number;
  colors?: string[];
  inkColor?: string;
  /** Ink for the figure printed inside each bar. */
  barInkColor?: string;
  labelColor?: string;
  /** Width reserved for the row labels, left of the bars. */
  labelWidth?: number;
  /** Big step caption in the bottom-right corner. */
  showStepLabel?: boolean;
  valueFormatter?: (value: number) => string;
  delayInFrames?: number;
  /** Frame the exit begins on. Omit to hold on the final standings. */
  exitAtInFrames?: number;
  /** Length of the exit. */
  exitInFrames?: number;
  /** Frame override — pass the parent frame inside a `<Sequence>`. */
  frame?: number;
};

const DEFAULT_COLORS = [
  "#e8b86d",
  "#2dd4bf",
  "#f472b6",
  "#8b8bf5",
  "#f59e0b",
  "#5eead4",
];

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

/** Value of a series at a fractional keyframe position. */
function valueAt(values: number[], time: number): number {
  if (values.length === 0) return 0;
  const clamped = Math.min(values.length - 1, Math.max(0, time));
  const lower = Math.floor(clamped);
  const upper = Math.min(values.length - 1, lower + 1);
  const t = clamped - lower;
  return values[lower] + (values[upper] - values[lower]) * t;
}

/**
 * Ranked bars that overtake each other as the clock runs.
 *
 * Rank is the sort position at each keyframe, eased between one keyframe and
 * the next. An integer rank of the live values makes bars teleport a full row
 * the instant two values cross; easing between keyframe orders slides them past
 * each other, and every bar rests on a whole row once the clock stops.
 *
 * Distinct from `animated-bar-chart`, which reveals a single fixed ranking.
 */
const BarChartRaceView: React.FC<BarChartRaceProps> = ({
  series,
  steps,
  framesPerStep = 26,
  visibleRows = 6,
  width = 900,
  rowHeight = 66,
  gap = 14,
  colors = DEFAULT_COLORS,
  inkColor = "#fafafa",
  barInkColor = "#0b0b10",
  labelColor = "rgba(250,250,250,0.55)",
  labelWidth = 200,
  showStepLabel = true,
  valueFormatter = (value: number) => formatCompactNumber(value),
  delayInFrames = 0,
  exitAtInFrames,
  exitInFrames = 20,
  frame: frameOverride,
}) => {
  const localFrame = useCurrentFrame();
  const frame = frameOverride ?? localFrame;

  const stepCount = series.reduce(
    (longest, entry) => Math.max(longest, entry.values.length),
    0,
  );
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

  const leader = current.reduce(
    (highest, entry) => (entry.present > 0 ? Math.max(highest, entry.current) : highest),
    0,
  ) || 1;

  const ranksAt = (step: number) => {
    const live = series
      .map((entry, index) => ({ index, value: entry.values[step], live: step < entry.values.length }))
      .filter((entry) => entry.live)
      .sort((a, b) => b.value - a.value || a.index - b.index);
    return series.map((_, index) => {
      const place = live.findIndex((entry) => entry.index === index);
      return place === -1 ? live.length : place;
    });
  };
  const lowerStep = Math.floor(time);
  const lowerRanks = ranksAt(lowerStep);
  const upperRanks = ranksAt(Math.min(lastStep, lowerStep + 1));
  const blend = time - lowerStep;
  const eased = blend * blend * (3 - 2 * blend);

  const exit =
    exitAtInFrames === undefined
      ? 0
      : interpolate(frame, [exitAtInFrames, exitAtInFrames + exitInFrames], [0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: EASING.exit,
        });

  const pitch = rowHeight + gap;
  const barWidth = width - labelWidth;
  const stepLabel = steps?.[Math.round(time)] ?? "";

  return (
    <div
      style={{
        position: "relative",
        width,
        height: pitch * visibleRows,
        opacity: 1 - clamp01(exit / 0.8),
      }}
    >
      {current.map((entry, index) => {
        const rank = lowerRanks[index] + (upperRanks[index] - lowerRanks[index]) * eased;

        // Rows past the visible window fade rather than clip, so a series
        // dropping off the board reads as losing rather than as a render bug.
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
                borderRadius: rowHeight * 0.22,
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
                {valueFormatter(entry.current)}
              </span>
            </div>
          </div>
        );
      })}

      {showStepLabel && stepLabel !== "" ? (
        <div
          style={{
            position: "absolute",
            right: 0,
            bottom: 0,
            color: inkColor,
            fontSize: rowHeight * 0.9,
            fontWeight: 700,
            opacity: 0.22,
            fontVariantNumeric: "tabular-nums",
          }}
        >
          {stepLabel}
        </div>
      ) : null}
    </div>
  );
};

export function BarChartRace(props: ElementProps & { children?: React.ReactNode }) {
  return <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}><BarChartRaceView {...(props as unknown as React.ComponentProps<typeof BarChartRaceView>)} /></AbsoluteFill>;
}
