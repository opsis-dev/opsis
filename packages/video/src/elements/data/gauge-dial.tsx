import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { arcPath, polar } from "#video/common/geometry.ts";
import { clamp01 } from "#video/common/math.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const EXIT_FRAMES = 20;

function decimalsOf(value: number): number {
  return Math.min(3, (String(value).split(".")[1] ?? "").length);
}

export function GaugeDial({
  value,
  min,
  max,
  size,
  thickness,
  sweepInDegrees: sweepSetting,
  color,
  trackColor,
  inkColor,
  labelColor,
  label,
  unit,
  tickCount,
  durationInFrames,
  roundEnds,
  showNeedle,
  uppercaseLabel,
  fontFamily,
  decimals,
  exitAtInFrames,
}: ElementSettings["gauge-dial"] & { exitAtInFrames?: number }) {
  const frame = useCurrentFrame();
  const delayInFrames = 0;
  const exitInFrames = EXIT_FRAMES;
  const cap = roundEnds ? "round" : "butt";

  const sweepInDegrees = Math.min(360, Math.max(1, sweepSetting));
  const fullCircle = sweepInDegrees >= 360;
  const tickSteps = fullCircle ? tickCount : tickCount - 1;
  function format(input: number) {
    return input.toFixed(decimals ?? decimalsOf(value));
  }
  const centre = size / 2;
  const radius = (size - thickness) / 2;
  const startAngle = -sweepInDegrees / 2;
  const endAngle = sweepInDegrees / 2;

  const span = max - min || 1;
  const target = clamp01((value - min) / span);

  const enterWindow: [number, number] = [delayInFrames, delayInFrames + durationInFrames];
  const arcProgress = interpolate(frame, enterWindow, [0, 1], { ...CLAMPED, easing: EASING.enter });
  const needleProgress = interpolate(frame, enterWindow, [0, 1], {
    ...CLAMPED,
    easing: EASING.pop,
  });

  const exit =
    exitAtInFrames === undefined
      ? 0
      : interpolate(frame, [exitAtInFrames, exitAtInFrames + exitInFrames], [0, 1], {
          ...CLAMPED,
          easing: EASING.exit,
        });

  const arcFraction = target * arcProgress * (1 - exit);
  const needleFraction = target * needleProgress * (1 - exit);
  const needleAngle = startAngle + needleFraction * sweepInDegrees;
  const needleTip = polar(centre, centre, radius - thickness * 0.55, needleAngle);
  const needleTail = polar(centre, centre, thickness * 0.5, needleAngle + 180);

  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <div
        style={{
          position: "relative",
          width: size,
          height: size,
          opacity: 1 - clamp01(exit / 0.85),
          fontFamily: fontOr(fontFamily, "main"),
        }}
      >
        <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
          <path
            d={arcPath(centre, centre, radius, startAngle, endAngle)}
            fill="none"
            stroke={trackColor}
            strokeWidth={thickness}
            strokeLinecap={cap}
          />
          <path
            d={arcPath(
              centre,
              centre,
              radius,
              startAngle,
              startAngle + arcFraction * sweepInDegrees,
            )}
            fill="none"
            stroke={color}
            strokeWidth={thickness}
            strokeLinecap={cap}
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

          {showNeedle ? (
            <>
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
            </>
          ) : null}
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
                letterSpacing: uppercaseLabel ? "0.06em" : undefined,
                textTransform: uppercaseLabel ? "uppercase" : undefined,
                marginTop: size * 0.028,
              }}
            >
              {label}
            </div>
          ) : null}
        </div>
      </div>
    </AbsoluteFill>
  );
}
