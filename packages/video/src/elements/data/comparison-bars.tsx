import type React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { type ElementProps } from "#video/common/spec.ts";
import { formatCompactNumber, readDelta } from "#video/lib/charts/chart-utils.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

export type ComparisonRowDatum = {
  label: string;
  before: number;
  after: number;
  delta?: string;
};

export type ComparisonBarsProps = {
  rows: ComparisonRowDatum[];
  width?: number;
  rowHeight?: number;
  gap?: number;
  labelWidth?: number;
  beforeColor?: string;
  afterColor?: string;
  downColor?: string;
  barInkColor?: string;
  labelColor?: string;
  seriesLabels?: [string, string];
  showDelta?: boolean;
  valueFormatter?: (value: number) => string;
  durationInFrames?: number;
  staggerInFrames?: number;
  pairOffsetInFrames?: number;
  delayInFrames?: number;
  exitAtInFrames?: number;
  exitInFrames?: number;
  frame?: number;
};

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

const ComparisonBarsView: React.FC<ComparisonBarsProps> = ({
  rows,
  width = 820,
  rowHeight = 74,
  gap = 26,
  labelWidth = 190,
  beforeColor = "rgba(250,250,250,0.22)",
  afterColor = "#e8b86d",
  downColor = "#f472b6",
  barInkColor = "#0b0b10",
  labelColor = "rgba(250,250,250,0.55)",
  seriesLabels,
  showDelta = true,
  valueFormatter = (value: number) => formatCompactNumber(value),
  durationInFrames = 30,
  staggerInFrames = 12,
  pairOffsetInFrames = 6,
  delayInFrames = 0,
  exitAtInFrames,
  exitInFrames = 20,
  frame: frameOverride,
}) => {
  const localFrame = useCurrentFrame();
  const frame = frameOverride ?? localFrame;

  const peak = rows.reduce((highest, row) => Math.max(highest, row.before, row.after), 0) || 1;
  const barWidth = width - labelWidth;
  const barHeight = (rowHeight - 8) / 2;
  const chipSize = barHeight * 0.5;
  const legend = [beforeColor, afterColor]
    .map((swatch, index) => ({ swatch, text: seriesLabels?.[index] ?? "" }))
    .filter((entry) => entry.text !== "");

  const exit =
    exitAtInFrames === undefined
      ? 0
      : interpolate(frame, [exitAtInFrames, exitAtInFrames + exitInFrames], [0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: EASING.exit,
        });

  const grow = (start: number) =>
    interpolate(frame, [start, start + durationInFrames], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: EASING.enter,
    });

  return (
    <div style={{ width, display: "grid", gap }}>
      {legend.length > 0 ? (
        <div
          style={{
            display: "flex",
            gap: gap,
            marginLeft: labelWidth,
            color: labelColor,
            fontSize: barHeight * 0.46,
            fontWeight: 600,
            opacity: 1 - clamp01(exit / 0.7),
          }}
        >
          {legend.map((entry, index) => (
            <span key={index} style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <span
                style={{
                  width: barHeight * 0.42,
                  height: barHeight * 0.42,
                  borderRadius: 4,
                  background: entry.swatch,
                }}
              />
              {entry.text}
            </span>
          ))}
        </div>
      ) : null}

      {rows.map((row, index) => {
        const start = delayInFrames + index * staggerInFrames;
        const beforeProgress = grow(start);
        const afterProgress = grow(start + pairOffsetInFrames);
        const drain = clamp01((exit - index * 0.05) / 0.7);

        const change = row.before === 0 ? 0 : ((row.after - row.before) / row.before) * 100;
        const delta =
          row.delta ??
          (row.before === 0 ? "" : `${change >= 0 ? "+" : "−"}${Math.abs(Math.round(change))}%`);
        const falls = row.delta ? readDelta(row.delta).direction === "down" : change < 0;

        return (
          <div
            key={index}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 0,
              opacity: 1 - drain,
              translate: `0 ${drain * rowHeight * 0.25}px`,
            }}
          >
            <span
              style={{
                width: labelWidth,
                paddingRight: 20,
                boxSizing: "border-box",
                color: labelColor,
                fontSize: barHeight * 0.5,
                fontWeight: 600,
                textAlign: "right",
              }}
            >
              {row.label}
            </span>

            <div style={{ display: "grid", gap: 8, flex: 1 }}>
              <div
                style={{
                  height: barHeight,
                  width: (row.before / peak) * barWidth * beforeProgress,
                  background: beforeColor,
                  borderRadius: barHeight * 0.28,
                }}
              />
              <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                <div
                  style={{
                    height: barHeight,
                    width: (row.after / peak) * barWidth * afterProgress,
                    background: afterColor,
                    borderRadius: barHeight * 0.28,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "flex-end",
                    paddingRight: barHeight * 0.32,
                    boxSizing: "border-box",
                  }}
                >
                  <span
                    style={{
                      color: barInkColor,
                      fontSize: barHeight * 0.46,
                      fontWeight: 700,
                      fontVariantNumeric: "tabular-nums",
                      opacity: clamp01((afterProgress - 0.55) / 0.25),
                    }}
                  >
                    {valueFormatter(row.after)}
                  </span>
                </div>

                {showDelta && delta ? (
                  <span
                    style={{
                      color: falls ? downColor : afterColor,
                      fontSize: chipSize,
                      fontWeight: 700,
                      fontVariantNumeric: "tabular-nums",
                      whiteSpace: "nowrap",
                      opacity: clamp01((afterProgress - 0.86) / 0.14),
                      translate: `${(1 - clamp01((afterProgress - 0.86) / 0.14)) * -12}px`,
                    }}
                  >
                    {delta}
                  </span>
                ) : null}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
};

export function ComparisonBars(props: ElementProps & { children?: React.ReactNode }) {
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <ComparisonBarsView
        {...(props as unknown as React.ComponentProps<typeof ComparisonBarsView>)}
      />
    </AbsoluteFill>
  );
}
