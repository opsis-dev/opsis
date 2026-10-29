import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { percentText } from "#video/common/format.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { formatNumber, readDelta } from "#video/lib/charts/chart-utils.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const EXIT_FRAMES = 20;

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

export function DonutChart({
  segments,
  size,
  thickness,
  colors,
  showLegend,
  showTotal,
  totalLabel,
  trackColor,
  labelColor,
  inkColor,
  upColor,
  downColor,
  durationInFrames,
  staggerInFrames,
  numberFormat,
  segmentGap,
  roundEnds,
  startAngle,
  uppercaseLabel,
  legendShows,
  fontFamily,
  exitAtInFrames,
}: ElementSettings["donut-chart"] & { exitAtInFrames?: number }) {
  const frame = useCurrentFrame();
  const delayInFrames = 0;
  const exitInFrames = EXIT_FRAMES;

  const total = segments.reduce((sum, segment) => sum + segment.value, 0) || 1;
  const radius = (size - thickness) / 2;
  const circumference = 2 * Math.PI * radius;
  const gap = Math.min((circumference * segmentGap) / 100, thickness * 0.22);

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
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          fontFamily: fontOr(fontFamily, "main"),
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
                  strokeLinecap={roundEnds ? "round" : "butt"}
                  strokeDasharray={`${length} ${circumference}`}
                  transform={`rotate(${-90 + startAngle + segment.start * 360} ${size / 2} ${size / 2})`}
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
                  {formatNumber(landed, numberFormat)}
                </div>
                <div
                  style={{
                    color: labelColor,
                    fontSize: size * 0.065,
                    fontWeight: 600,
                    letterSpacing: uppercaseLabel ? "0.06em" : undefined,
                    textTransform: uppercaseLabel ? "uppercase" : undefined,
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
                  {legendShows === "percent"
                    ? percentText(segment.fraction * segment.progress)
                    : formatNumber(segment.value * segment.progress, numberFormat)}
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
    </AbsoluteFill>
  );
}
