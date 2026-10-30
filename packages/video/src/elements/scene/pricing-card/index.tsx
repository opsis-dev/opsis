import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { codePalette } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const T = {
  card: 0,
  cardFor: 0.48,
  head: 0.16,
  headFor: 0.4,
  ctaAfterFeatures: 0.16,
  ctaFor: 0.42,
  exitFor: 0.42,
} as const;

export function PricingCard({
  tier,
  badge,
  price,
  currency,
  period,
  note,
  wasPrice,
  features,
  ctaLabel,
  priceAtSeconds,
  rollSeconds,
  featuresAtSeconds,
  featureStaggerSeconds,
  holdSeconds,
  accentColor,
  backgroundColor,
  theme,
  speed,
  ctaTextColor,
  rollFrom,
  cardWidth,
  fontFamily,
}: ElementSettings["pricing-card"] & { holdSeconds?: number }) {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const palette = codePalette(theme);
  const safe = getSafeAreaPadding({ width, height });

  function at(seconds: number) {
    return (seconds * fps) / speed;
  }
  function ease(from: number, to: number, easing = EASING.enter) {
    return interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...CLAMPED });
  }

  const portrait = height > width;
  const u = portrait ? Math.min(width / 496, height / 896) : Math.min(width / 1024, height / 576);

  const cardIn = spring({
    frame: frame - at(T.card),
    fps,
    config: { damping: 18, stiffness: 130, mass: 0.85 },
  });
  const open = ease(T.card, T.card + T.cardFor, EASING.editorial);
  const headIn = ease(T.head, T.head + T.headFor);

  const roll = ease(priceAtSeconds, priceAtSeconds + rollSeconds, EASING.editorial);
  const priceDecimals = Math.min(2, (String(price).split(".")[1] ?? "").length);
  const shown = interpolate(roll, [0, 1], [rollFrom, price]).toFixed(priceDecimals);
  const rollBlur = Math.sin(Math.PI * roll) * 2.2;

  const lastFeatureAt =
    featuresAtSeconds + Math.max(0, features.length - 1) * featureStaggerSeconds;
  const ctaAt = lastFeatureAt + T.ctaAfterFeatures;
  const ctaIn = ease(ctaAt, ctaAt + T.ctaFor, EASING.pop);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...CLAMPED,
        });

  const cardW = Math.min(width - safe.paddingLeft - safe.paddingRight, cardWidth * u);
  const pad = 30 * u;

  return (
    <AbsoluteFill
      style={{
        background: backgroundColor || palette.page,
        fontFamily: fontOr(fontFamily, "main"),
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: `${safe.paddingTop}px ${safe.paddingRight}px`,
      }}
    >
      <div
        style={{
          width: cardW,
          borderRadius: 24 * u,
          background: palette.window,
          border: `1px solid color-mix(in srgb, ${accentColor} 23%, transparent)`,
          boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 ${28 * u}px ${
            70 * u
          }px ${palette.shadow}`,
          padding: pad,
          position: "relative",
          overflow: "hidden",
          clipPath: `inset(0 0 ${(1 - open) * 100}% 0 round ${24 * u}px)`,
          opacity: 1 - exit,
          translate: `0 ${(1 - cardIn) * 24 * u + exit * 32 * u}px`,
        }}
      >
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: `radial-gradient(120% 80% at 8% 0%, color-mix(in srgb, ${accentColor} 12%, transparent), transparent 62%)`,
          }}
        />

        <div style={{ position: "relative" }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10 * u,
              opacity: headIn,
              translate: `0 ${(1 - headIn) * 10 * u}px`,
            }}
          >
            <span
              style={{
                color: palette.fg,
                fontSize: 19 * u,
                fontWeight: 700,
                letterSpacing: "0.02em",
              }}
            >
              {tier}
            </span>
            {badge ? (
              <span
                style={{
                  padding: `${3.5 * u}px ${9 * u}px`,
                  borderRadius: 999,
                  background: `color-mix(in srgb, ${accentColor} 13%, transparent)`,
                  border: `1px solid color-mix(in srgb, ${accentColor} 40%, transparent)`,
                  color: accentColor,
                  fontSize: 14.5 * u,
                  fontWeight: 600,
                }}
              >
                {badge}
              </span>
            ) : null}
          </div>

          <div
            style={{
              marginTop: 16 * u,
              display: "flex",
              alignItems: "baseline",
              gap: 8 * u,
              filter: rollBlur > 0.05 ? `blur(${rollBlur}px)` : undefined,
            }}
          >
            <span
              style={{
                color: palette.dim,
                fontSize: 30 * u,
                fontWeight: 600,
                opacity: roll > 0 ? 1 : 0,
              }}
            >
              {currency}
            </span>
            <span
              style={{
                color: palette.fg,
                fontSize: 68 * u,
                fontWeight: 800,
                letterSpacing: "-0.04em",
                lineHeight: 1,
                fontVariantNumeric: "tabular-nums",
                translate: `0 ${(1 - roll) * 10 * u}px`,
                opacity: roll > 0 ? 1 : 0,
              }}
            >
              {shown}
            </span>
            {period ? (
              <span
                style={{
                  color: palette.dim,
                  fontSize: 19 * u,
                  fontWeight: 600,
                  opacity: interpolate(roll, [0.5, 1], [0, 1], CLAMPED),
                }}
              >
                {period}
              </span>
            ) : null}
            {wasPrice !== undefined ? (
              <span
                style={{
                  marginLeft: 4 * u,
                  color: palette.faint,
                  fontSize: 20 * u,
                  fontWeight: 600,
                  textDecoration: "line-through",
                  fontVariantNumeric: "tabular-nums",
                  opacity: interpolate(roll, [0.7, 1], [0, 1], CLAMPED),
                }}
              >
                {currency}
                {wasPrice}
              </span>
            ) : null}
          </div>

          {note ? (
            <div
              style={{
                marginTop: 6 * u,
                color: palette.dim,
                fontSize: 14.5 * u,
                fontWeight: 500,
                opacity: interpolate(roll, [0.6, 1], [0, 1], CLAMPED),
              }}
            >
              {note}
            </div>
          ) : null}

          <div
            style={{
              marginTop: 20 * u,
              paddingTop: 18 * u,
              borderTop: `1px solid ${palette.border}`,
              display: "flex",
              flexDirection: "column",
              gap: 12 * u,
            }}
          >
            {features.map((feature, index) => {
              const featureAt = featuresAtSeconds + index * featureStaggerSeconds;
              const featureIn = ease(featureAt, featureAt + 0.38);
              return (
                <div
                  key={index}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 11 * u,
                    opacity: featureIn,
                    translate: `${(1 - featureIn) * -10 * u}px`,
                  }}
                >
                  <svg width={18 * u} height={18 * u} viewBox="0 0 24 24" fill="none">
                    <circle
                      cx="12"
                      cy="12"
                      r="11"
                      fill={`color-mix(in srgb, ${accentColor} 13%, transparent)`}
                    />
                    <path
                      d="M6.5 12.5 L10.5 16.5 L17.5 8"
                      stroke={accentColor}
                      strokeWidth={2.6}
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      pathLength={1}
                      strokeDasharray={1}
                      strokeDashoffset={1 - featureIn}
                    />
                  </svg>
                  <span
                    style={{
                      color: palette.fg,
                      fontSize: 15.5 * u,
                      fontWeight: 500,
                    }}
                  >
                    {feature}
                  </span>
                </div>
              );
            })}
          </div>

          {ctaLabel ? (
            <div
              style={{
                marginTop: 22 * u,
                height: 48 * u,
                borderRadius: 12 * u,
                background: accentColor,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: ctaTextColor || (theme.mode === "dark" ? "#141013" : "#1B1509"),
                fontSize: 16.5 * u,
                fontWeight: 700,
                letterSpacing: "0.01em",
                opacity: Math.min(1, ctaIn * 1.4),
                scale: `${0.94 + ctaIn * 0.06}`,
                boxShadow: `0 ${10 * u}px ${26 * u}px color-mix(in srgb, ${accentColor} 20%, transparent)`,
              }}
            >
              {ctaLabel}
            </div>
          ) : null}
        </div>
      </div>
    </AbsoluteFill>
  );
}
