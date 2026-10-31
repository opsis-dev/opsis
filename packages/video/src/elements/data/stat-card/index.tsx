import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { spring } from "@opsis/runtime/spring.ts";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings, GroupSeparator } from "#video/generated/spec.ts";
import { groupDigits } from "#video/lib/charts/chart-utils.ts";
import { CODE_THEMES } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const PALETTES = {
  dark: {
    page: CODE_THEMES.dark.page,
    fg: CODE_THEMES.dark.fg,
    faint: CODE_THEMES.dark.faint,
    band: CODE_THEMES.dark.band,
  },
  light: {
    page: CODE_THEMES.light.page,
    fg: CODE_THEMES.light.fg,
    faint: CODE_THEMES.light.faint,
    band: CODE_THEMES.light.band,
  },
};

function formatValue(value: number, decimals: number, separator: GroupSeparator): string {
  const [whole, fraction] = value.toFixed(decimals).split(".");
  const grouped = groupDigits(whole, separator);
  return fraction === undefined ? grouped : `${grouped}.${fraction}`;
}

const T = {
  ring: 0.1,
  ringFor: 1.5,
  count: 0.16,
  countFor: 2.2,
  label: 0.6,
  caption: 0.78,
  delta: 2.7,
  exitFor: 0.4,
} as const;

export function StatCard({
  value,
  max,
  suffix,
  prefix,
  decimals,
  label,
  caption,
  delta: deltaSetting,
  showDelta,
  deltaSuffix,
  backgroundColor,
  accentColor,
  theme,
  speed,
  glow,
  ringThickness,
  groupSeparator,
  fontFamily,
  downColor,
  ringGlow,
  holdSeconds,
}: ElementSettings["stat-card"] & { holdSeconds?: number }) {
  const frame = useFrame();
  const { fps, width, height } = useFilm();
  const delta = showDelta ? deltaSetting : undefined;
  const deltaColor = delta !== undefined && delta < 0 ? downColor || accentColor : accentColor;
  const palette = { ...PALETTES[theme.mode], ...theme.colors };
  const safe = getSafeAreaPadding({ width, height });

  function at(seconds: number) {
    return (seconds * fps) / speed;
  }
  function ease(from: number, to: number, easing = EASING.enter) {
    return interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...CLAMPED });
  }
  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...CLAMPED,
        });

  const stage = {
    w: width - safe.paddingLeft - safe.paddingRight,
    h: height - safe.paddingTop - safe.paddingBottom,
  };
  const portrait = height > width;
  const u = portrait
    ? Math.min(stage.w / 620, stage.h / 1120)
    : Math.min(stage.w / 1120, stage.h / 620);

  const share = max && max > 0 ? Math.min(Math.max(value / max, 0), 1) : 1;
  const ring = ease(T.ring, T.ring + T.ringFor, EASING.enter) * share;
  const counted = ease(T.count, T.count + T.countFor, EASING.enter) * value;
  const labelIn = ease(T.label, T.label + 0.45);
  const captionIn = caption ? ease(T.caption, T.caption + 0.45) : 0;
  const deltaIn =
    delta === undefined
      ? 0
      : spring({
          frame: frame - at(T.delta),
          fps,
          config: { damping: 13, stiffness: 190, mass: 0.6 },
        });
  const land = interpolate(
    frame,
    [at(T.ring + T.ringFor), at(T.ring + T.ringFor + 0.45)],
    [1, 0],
    CLAMPED,
  );

  const size = Math.min(stage.w * 0.62, stage.h * 0.62, 380 * u);
  const stroke = ringThickness * u;
  const radius = size / 2 - stroke;
  const circumference = 2 * Math.PI * radius;

  return (
    <div
      style={{
        width,
        height,
        background: backgroundColor || palette.page,
        fontFamily: fontOr(fontFamily, "main"),
        position: "relative",
        overflow: "hidden",
        display: "grid",
        placeItems: "center",
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `radial-gradient(ellipse 55% 50% at 50% 45%, color-mix(in srgb, ${accentColor} ${glow}%, transparent), transparent 72%)`,
        }}
      />

      <div
        style={{
          position: "relative",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 18 * u,
          opacity: 1 - exit,
          translate: `0 ${exit * -26 * u}px`,
        }}
      >
        <div style={{ position: "relative", width: size, height: size }}>
          <svg width={size} height={size} style={{ rotate: "-90deg" }}>
            <circle
              cx={size / 2}
              cy={size / 2}
              r={radius}
              fill="none"
              stroke={palette.band}
              strokeWidth={stroke}
            />
            <circle
              cx={size / 2}
              cy={size / 2}
              r={radius}
              fill="none"
              stroke={accentColor}
              strokeWidth={stroke}
              strokeLinecap="round"
              strokeDasharray={circumference}
              strokeDashoffset={circumference * (1 - ring)}
              style={{
                filter: `drop-shadow(0 0 ${(ringGlow + land * 16) * u}px ${accentColor})`,
              }}
            />
          </svg>

          <div
            style={{
              position: "absolute",
              inset: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: palette.fg,
              fontSize: 92 * u,
              fontWeight: 700,
              letterSpacing: "-0.03em",
              fontVariantNumeric: "tabular-nums",
              scale: interpolate(land, [0, 1], [1, 1.03], {
                output: "perceptual-scale",
                ...CLAMPED,
              }),
            }}
          >
            {prefix ? (
              <span style={{ fontSize: 54 * u, alignSelf: "flex-start", marginTop: 20 * u }}>
                {prefix}
              </span>
            ) : null}
            {formatValue(counted, decimals, groupSeparator)}
            {suffix ? <span style={{ fontSize: 54 * u, color: accentColor }}>{suffix}</span> : null}
          </div>
        </div>

        {label ? (
          <div
            style={{
              color: palette.fg,
              fontSize: 34 * u,
              fontWeight: 600,
              letterSpacing: "-0.01em",
              opacity: labelIn,
              translate: `0 ${(1 - labelIn) * 12 * u}px`,
            }}
          >
            {label}
          </div>
        ) : null}

        {caption ? (
          <div
            style={{
              marginTop: -8 * u,
              color: palette.faint,
              fontSize: 21 * u,
              opacity: captionIn,
            }}
          >
            {caption}
          </div>
        ) : null}

        {delta !== undefined ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8 * u,
              padding: `${8 * u}px ${15 * u}px`,
              borderRadius: 999,
              background: `color-mix(in srgb, ${deltaColor} 9%, transparent)`,
              border: `1px solid color-mix(in srgb, ${deltaColor} 33%, transparent)`,
              color: deltaColor,
              fontSize: 22 * u,
              fontWeight: 600,
              fontVariantNumeric: "tabular-nums",
              opacity: Math.min(deltaIn, 1),
              scale: interpolate(Math.min(deltaIn, 1), [0, 1], [0.86, 1], {
                output: "perceptual-scale",
              }),
            }}
          >
            <svg width={16 * u} height={16 * u} viewBox="0 0 16 16" fill="none">
              <path
                d={delta < 0 ? "M8 3v10m0 0 4-4m-4 4-4-4" : "M8 13V3m0 0 4 4M8 3 4 7"}
                stroke={deltaColor}
                strokeWidth={2}
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
            {delta > 0 ? "+" : ""}
            {delta}
            {deltaSuffix}
          </div>
        ) : null}
      </div>
    </div>
  );
}
