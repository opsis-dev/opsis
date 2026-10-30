import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { StoryLook } from "#video/elements/scene/data-story/look.ts";
import type { NumberFormat } from "#video/generated/spec.ts";
import {
  formatAxisValue,
  formatNumber,
  niceDomain,
  readDelta,
  type ChartDatum,
} from "#video/lib/charts/chart-utils.ts";
import { getSafeAreaPadding, scaleFont } from "#video/lib/layout/layout.ts";
import { DURATION, EASING, STAGGER } from "#video/lib/motion/motion-tokens.ts";

export type AnimatedBarChartProps = {
  data: ChartDatum[];
  title?: string;
  subtitle?: string;
  maxValue?: number;
  numberFormat: NumberFormat;
  highlightLabel?: string;
  showAxis: boolean;
  maxBars?: number;
  barColor: string;
  accentColor: string;
  backgroundColor: string;
  look: StoryLook;
  holdSeconds?: number;
};

const EXIT_FOR = 0.42;

export function AnimatedBarChart({
  data,
  title,
  subtitle,
  maxValue,
  numberFormat,
  highlightLabel,
  showAxis,
  maxBars = 6,
  barColor,
  accentColor,
  backgroundColor,
  look,
  holdSeconds,
}: AnimatedBarChartProps) {
  const COLORS = look.palette;
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const safe = getSafeAreaPadding({ width, height });
  const isPortrait = height > width;

  const bars = data.slice(0, maxBars);
  const domain = niceDomain(
    maxValue === undefined ? bars.map((item) => item.value) : [0, maxValue],
    { includeZero: true, tickCount: isPortrait ? 3 : 4 },
  );

  const gap = scaleFont(16, width);
  const longestLabel = bars.reduce((longest, item) => Math.max(longest, item.label.length), 0);
  const labelSize = scaleFont(isPortrait ? 30 : 28, width);
  const valueSize = scaleFont(isPortrait ? 32 : 30, width);
  const labelWidth = Math.min(
    Math.round(width * 0.26),
    Math.round(Math.min(longestLabel, 14) * labelSize * 0.62) + gap,
  );
  const valueWidth = Math.round(scaleFont(96, width));
  const barHeight = Math.min(
    scaleFont(52, width),
    Math.round((height * (isPortrait ? 0.44 : 0.52)) / Math.max(1, bars.length)),
  );

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [holdSeconds * fps, (holdSeconds + EXIT_FOR) * fps], [0, 1], {
          ...CLAMPED,
          easing: EASING.exit,
        });
  const leaving = {
    opacity: 1 - exit,
    translate: `0 ${exit * -scaleFont(26, width)}px`,
  };

  const headerProgress = interpolate(frame, [0, DURATION.normal], [0, 1], {
    ...CLAMPED,
    easing: EASING.enter,
  });
  const subtitleProgress = interpolate(
    frame,
    [STAGGER.tight, STAGGER.tight + DURATION.normal],
    [0, 1],
    { ...CLAMPED, easing: EASING.enter },
  );
  const axisProgress = interpolate(
    frame,
    [STAGGER.normal, STAGGER.normal + DURATION.normal],
    [0, 1],
    { ...CLAMPED, easing: EASING.enter },
  );

  return (
    <div
      style={{
        width,
        height,
        background: backgroundColor,
        color: COLORS.ink,
        fontFamily: look.font,
        paddingLeft: safe.paddingLeft,
        paddingRight: safe.paddingRight,
        paddingTop: safe.paddingTop,
        paddingBottom: safe.paddingBottom,
        display: "flex",
        flexDirection: "column",
        justifyContent: isPortrait ? "center" : "flex-start",
        gap: scaleFont(40, width),
      }}
    >
      {title ? (
        <header style={{ display: "grid", gap: scaleFont(12, width), ...leaving }}>
          <h2
            style={{
              margin: 0,
              fontSize: scaleFont((isPortrait ? 76 : 64) * look.textSize, width),
              fontWeight: 700,
              lineHeight: 1.02,
              letterSpacing: "-0.025em",
              opacity: headerProgress,
              translate: `0 ${(1 - headerProgress) * scaleFont(18, width)}px`,
            }}
          >
            {title}
          </h2>
          {subtitle ? (
            <p
              style={{
                margin: 0,
                color: COLORS.label,
                fontSize: scaleFont(30, width),
                fontWeight: 500,
                lineHeight: 1.25,
                opacity: subtitleProgress,
                translate: `0 ${(1 - subtitleProgress) * scaleFont(12, width)}px`,
              }}
            >
              {subtitle}
            </p>
          ) : null}
        </header>
      ) : null}

      <div
        style={{
          flex: isPortrait ? "0 0 auto" : 1,
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          ...leaving,
        }}
      >
        <div
          style={{
            position: "relative",
            display: "flex",
            flexDirection: "column",
            gap: scaleFont(18, width),
          }}
        >
          {showAxis ? (
            <div
              style={{
                position: "absolute",
                left: labelWidth + gap,
                right: valueWidth + gap,
                top: 0,
                bottom: 0,
                opacity: axisProgress,
              }}
            >
              {domain.ticks.map((tick) => (
                <div
                  key={tick}
                  style={{
                    position: "absolute",
                    top: 0,
                    bottom: 0,
                    left: `${((tick - domain.min) / domain.span) * 100}%`,
                    width: 1,
                    background: COLORS.grid,
                  }}
                />
              ))}
            </div>
          ) : null}

          {bars.map((item, index) => {
            const delay = STAGGER.normal + index * STAGGER.normal;
            const enter = spring({
              frame: frame - delay,
              fps,
              config: { damping: 20, stiffness: 110, mass: 0.9 },
              durationInFrames: DURATION.slow,
            });
            const isHighlighted = highlightLabel === item.label;
            const fill = item.color ?? (isHighlighted ? accentColor : barColor);
            const ratio = Math.max(0, (item.value - domain.min) / domain.span);
            const delta = readDelta(item.delta);

            return (
              <div
                key={item.label}
                style={{
                  display: "grid",
                  gridTemplateColumns: `${labelWidth}px 1fr ${valueWidth}px`,
                  gap,
                  alignItems: "center",
                  opacity: Math.min(1, enter * 1.6),
                  translate: `0 ${(1 - enter) * scaleFont(14, width)}px`,
                }}
              >
                <div
                  style={{
                    color: isHighlighted ? COLORS.ink : COLORS.label,
                    fontSize: labelSize,
                    fontWeight: isHighlighted ? 700 : 600,
                    letterSpacing: "-0.01em",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {item.label}
                </div>

                <div
                  style={{
                    position: "relative",
                    height: barHeight,
                    borderRadius: barHeight / 2,
                    background: COLORS.track,
                  }}
                >
                  <div
                    style={{
                      width: `${Math.max(ratio, 0.015) * enter * 100}%`,
                      height: "100%",
                      borderRadius: barHeight / 2,
                      background: `linear-gradient(90deg, color-mix(in srgb, ${fill} 90%, transparent) 0%, ${fill} 62%)`,
                      boxShadow: isHighlighted
                        ? `0 0 ${scaleFont(28, width)}px color-mix(in srgb, ${fill} 27%, transparent)`
                        : undefined,
                    }}
                  />
                </div>

                <div
                  style={{
                    display: "grid",
                    justifyItems: "end",
                    gap: scaleFont(4, width),
                  }}
                >
                  <div
                    style={{
                      fontSize: valueSize,
                      fontWeight: 700,
                      letterSpacing: "-0.02em",
                      lineHeight: 1,
                      fontVariantNumeric: "tabular-nums",
                      color: isHighlighted ? fill : COLORS.ink,
                    }}
                  >
                    {formatNumber(item.value * enter, numberFormat)}
                  </div>
                  {delta.text ? (
                    <div
                      style={{
                        fontSize: scaleFont(21, width),
                        fontWeight: 600,
                        lineHeight: 1,
                        color:
                          delta.direction === "down"
                            ? COLORS.down
                            : delta.direction === "up"
                              ? COLORS.up
                              : COLORS.label,
                        opacity: interpolate(enter, [0.7, 1], [0, 1], CLAMPED),
                      }}
                    >
                      {delta.text}
                    </div>
                  ) : null}
                </div>
              </div>
            );
          })}
        </div>

        {showAxis ? (
          <div
            style={{
              position: "relative",
              height: scaleFont(24, width),
              marginLeft: labelWidth + gap,
              marginRight: valueWidth + gap,
              marginTop: scaleFont(10, width),
              opacity: axisProgress,
            }}
          >
            {domain.ticks.map((tick, index) => (
              <div
                key={tick}
                style={{
                  position: "absolute",
                  top: 0,
                  left: `${((tick - domain.min) / domain.span) * 100}%`,
                  transform:
                    index === 0
                      ? "none"
                      : index === domain.ticks.length - 1
                        ? "translateX(-100%)"
                        : "translateX(-50%)",
                  color: COLORS.axis,
                  fontSize: scaleFont(21, width),
                  fontWeight: 600,
                  fontVariantNumeric: "tabular-nums",
                  whiteSpace: "nowrap",
                }}
              >
                {formatAxisValue(tick)}
              </div>
            ))}
          </div>
        ) : null}
      </div>
    </div>
  );
}
