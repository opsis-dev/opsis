import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { Img } from "@opsis/runtime/parts.tsx";
import { spring } from "@opsis/runtime/spring.ts";
import type React from "react";
import { isTheme, themeColors } from "#video/common/theme.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { interpolateColors, parseColor, rgbToOklch, withAlpha } from "#video/lib/ui-kit/color.ts";
import { resolveFont, type WarmTheme, useWarmTheme } from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

function lighterOf(a: string, b: string): string {
  return (rgbToOklch(parseColor(a)).l ?? 0) >= (rgbToOklch(parseColor(b)).l ?? 0) ? a : b;
}

function strikethroughPhases(
  startAt: number,
  drawDuration: number,
): { drawStart: number; drawEnd: number; fadeStart: number; fadeEnd: number } {
  const drawEnd = startAt + drawDuration;
  return {
    drawStart: startAt,
    drawEnd,
    fadeStart: drawEnd,
    fadeEnd: drawEnd + drawDuration * 0.5,
  };
}

function perspectiveScale(travel: number, maxScale: number): number {
  if (maxScale <= 1) return 1;
  const p = 1 - 1 / maxScale;
  return 1 / (1 - p * Math.min(Math.max(travel, 0), 0.9999));
}

function swingWorldX(travel: number, drift: number, out: number): number {
  return drift * travel - out * Math.sin(Math.PI * travel);
}

function swingGain(maxScale: number): number {
  let peak = 1;
  for (let i = 1; i < 128; i++) {
    const t = i / 128;
    const v = Math.sin(Math.PI * t) * perspectiveScale(t, maxScale);
    if (v > peak) peak = v;
  }
  return peak;
}

function inkReach(width: number, height: number, driftPx: number): number {
  const nearest = driftPx;
  const furthest = width / 2 + driftPx;
  const horizontal = Math.max(
    Math.abs(nearest),
    Math.abs(furthest),
    Math.abs(width - nearest),
    Math.abs(width - furthest),
  );
  return Math.hypot(horizontal, height / 2) * 1.05;
}

const INK_LAG = 0.7;

const WIPE_EASE = Easing.bezier(0.2, 0.6, 0.35, 1);
const RUSH_EASE = Easing.bezier(0.45, 0.2, 0.45, 1);

function SnapMark({ size, color }: { size: number; color: string }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 464 409"
      fill="none"
      aria-hidden="true"
      style={{ display: "block" }}
    >
      <title>mark</title>
      <g transform="translate(0,409) scale(0.1,-0.1)">
        <path
          d="M589 3805 c-167 -41 -305 -149 -380 -297 -60 -120 -59 -84 -59 -1239 0 -1004 1 -1059 19 -1112 48 -147 148 -226 331 -262 100 -20 127 -33 186 -87 53 -49 74 -109 74 -212 1 -121 42 -196 133 -244 l52 -27 980 -3 c883 -3 986 -1 1042 13 75 21 133 67 171 139 l27 51 3 315 c2 173 2 342 0 374 l-3 60 -31 -50 c-36 -59 -101 -115 -170 -146 l-49 -23 -810 -3 c-591 -2 -825 0 -865 8 -128 28 -258 121 -325 234 -65 110 -66 128 -63 826 l3 625 24 58 c44 111 147 210 270 259 l56 23 830 3 c809 2 831 2 895 -18 80 -25 149 -74 198 -142 l37 -52 3 334 c3 367 -3 418 -56 494 -30 44 -98 91 -152 106 -22 6 -457 10 -1175 9 -923 -1 -1151 -3 -1196 -14z M3800 2983 c-327 -196 -598 -361 -602 -367 -4 -6 -7 -254 -5 -550 l2 -540 145 -91 c80 -50 208 -130 285 -177 77 -47 283 -174 457 -282 174 -108 326 -196 337 -196 11 0 27 7 35 16 14 14 16 141 16 1254 0 781 -4 1248 -10 1264 -7 20 -16 26 -37 26 -18 -1 -232 -123 -623 -357z"
          fill={color}
        />
      </g>
    </svg>
  );
}

function TextHighlightView({
  before,
  highlight,
  after,
  preset,
  baseColor,
  accentColor,
  background,
  colors,
  mode,
  fontFamily,
  highlightedTextColor,
  replaceWith,
  startAt,
  drawDuration,
  thickness,
  shineColor,
  logoSrc,
  logoScale,
  spinTurns,
  holdDuration,
  rushDuration,
  rushScale,
  rushDrift,
  swingOut,
  coverColor,
  fontSize,
  fontWeight,
  speed,
  letterSpacing,
  lineHeight,
  textWidth,
  markerOpacity,
  shineWidth,
  markGap,
}: Omit<ElementSettings["text-highlight"], "theme"> & {
  colors?: Partial<WarmTheme>;
  mode?: "light" | "dark";
}) {
  const frame = useAuthoredFrame() * speed;
  const { fps, width, height } = useAuthoredConfig();
  const t = useWarmTheme(colors, mode);
  const ground = background ?? t.background;
  const face =
    resolveFont(fontFamily || t.fontFamily) ??
    "Inter, -apple-system, BlinkMacSystemFont, sans-serif";
  const base = baseColor ?? t.foreground;
  const accent = accentColor ?? t.primary;
  const shine = shineColor || lighterOf(t.card, t.foreground);

  const emphasisColor =
    highlightedTextColor ?? (preset === "color" || preset === "strikethrough" ? accent : base);
  const lineThickness = thickness;

  const sweep = spring({
    frame: frame - startAt,
    fps,
    config: { damping: 18 },
    durationInFrames: drawDuration,
  });

  const textStyle: React.CSSProperties = {
    fontSize,
    fontWeight,
    color: base,
    letterSpacing: `${letterSpacing}em`,
    lineHeight,
    fontFamily: face,
    whiteSpace: "pre-wrap",
    textAlign: "center",
    maxWidth: `${textWidth}%`,
  };

  if (preset === "logo-wipe") {
    const wipe = interpolate(frame, [startAt, startAt + drawDuration], [0, 1], {
      ...CLAMPED,
      easing: WIPE_EASE,
    });
    const edge = (1 - wipe) * 100;

    const rushStart = startAt + drawDuration + holdDuration;
    const travel = interpolate(frame, [rushStart, rushStart + rushDuration], [0, 1], {
      ...CLAMPED,
      easing: RUSH_EASE,
    });

    const markSize = fontSize * logoScale;
    const rushed = perspectiveScale(travel, rushScale);
    const markBox = markSize * rushed;
    const diagonal = Math.hypot(width, height);

    const worldDrift = (rushDrift * width) / rushScale;
    const worldBow = (swingOut * width) / swingGain(rushScale);
    const swingPx = rushed * swingWorldX(travel, worldDrift, worldBow);

    const flood = interpolate(markBox, [diagonal * 0.55, diagonal * 1.45], [0, 1], CLAMPED);
    const inkPx = swingPx * (1 - INK_LAG);
    const floodRadius =
      flood <= 0
        ? 0
        : interpolate(
            flood,
            [0, 1],
            [markBox * 0.03, inkReach(width, height, rushDrift * width * (1 - INK_LAG))],
          );

    return (
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: ground,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          overflow: "visible",
        }}
      >
        <div
          style={{
            position: "relative",
            paddingLeft: markSize + fontSize * markGap,
          }}
        >
          <span
            style={{
              ...textStyle,
              position: "relative",
              display: "inline-block",
              maxWidth: undefined,
              whiteSpace: "nowrap",
            }}
          >
            <span
              style={{
                display: "block",
                clipPath: `inset(0 0 0 ${edge}%)`,
                backgroundImage: `linear-gradient(90deg, ${accent}, ${emphasisColor})`,
                backgroundClip: "text",
                WebkitBackgroundClip: "text",
                color: "transparent",
              }}
            >
              {highlight}
            </span>

            <div
              aria-hidden="true"
              style={{
                position: "absolute",
                top: "50%",
                left: `${edge}%`,
                width: "max-content",
                translate: `calc(-100% - ${fontSize * 0.16}px + ${swingPx}px) -50%`,
              }}
            >
              <div
                aria-hidden="true"
                style={{
                  position: "absolute",
                  left: "50%",
                  top: "50%",
                  translate: `calc(-50% - ${swingPx - inkPx}px) -50%`,
                  borderRadius: "50%",
                  background: coverColor || accent,
                  width: 2 * floodRadius,
                  height: 2 * floodRadius,
                }}
              />
              <div
                style={{
                  transformOrigin: "50% 50%",
                  rotate: `${spinTurns * 360 * wipe + 30 * travel}deg`,
                  scale: `${interpolate(wipe, [0, 1], [0.55, 1], CLAMPED) * rushed}`,
                }}
              >
                {logoSrc ? (
                  <Img
                    src={logoSrc}
                    alt=""
                    style={{
                      display: "block",
                      width: markSize,
                      height: markSize,
                      maxWidth: "none",
                      objectFit: "contain",
                    }}
                  />
                ) : (
                  <SnapMark size={markSize} color={accent} />
                )}
              </div>
            </div>
          </span>
        </div>
      </div>
    );
  }

  let highlightNode: React.ReactNode;

  if (preset === "marker") {
    highlightNode = (
      <span style={{ position: "relative", display: "inline-block" }}>
        <span
          aria-hidden
          style={{
            position: "absolute",
            inset: "-0.02em -0.12em",
            background: withAlpha(accent, markerOpacity / 100),
            borderRadius: "0.12em",
            transformOrigin: "left center",
            transform: `scaleX(${sweep})`,
            zIndex: 0,
          }}
        />
        <span
          style={{
            position: "relative",
            zIndex: 1,
            color: interpolateColors(
              interpolate(sweep, [0.5, 0.85], [0, 1], CLAMPED),
              [0, 1],
              [base, emphasisColor],
            ),
          }}
        >
          {highlight}
        </span>
      </span>
    );
  } else if (preset === "color") {
    highlightNode = (
      <span
        style={{
          color: interpolateColors(
            interpolate(frame, [startAt, startAt + drawDuration], [0, 1], CLAMPED),
            [0, 1],
            [base, emphasisColor],
          ),
        }}
      >
        {highlight}
      </span>
    );
  } else if (preset === "underline") {
    highlightNode = (
      <span style={{ position: "relative", display: "inline-block" }}>
        <span
          style={{
            color: interpolateColors(
              interpolate(sweep, [0.5, 0.85], [0, 1], CLAMPED),
              [0, 1],
              [base, emphasisColor],
            ),
          }}
        >
          {highlight}
        </span>
        <span
          aria-hidden
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            bottom: "-0.06em",
            height: lineThickness,
            background: accent,
            borderRadius: lineThickness / 2,
            transformOrigin: "left center",
            transform: `scaleX(${sweep})`,
          }}
        />
      </span>
    );
  } else if (preset === "strikethrough") {
    const phases = strikethroughPhases(startAt, drawDuration);
    highlightNode = (
      <span
        style={{
          display: "inline-grid",
          verticalAlign: "baseline",
          justifyItems: "center",
        }}
      >
        <span
          style={{
            gridArea: "1 / 1",
            position: "relative",
            whiteSpace: "pre",
            opacity: interpolate(frame, [phases.fadeStart, phases.fadeEnd], [1, 0], CLAMPED),
          }}
        >
          {highlight}
          <span
            aria-hidden
            style={{
              position: "absolute",
              left: 0,
              top: "50%",
              height: lineThickness,
              width: `${interpolate(frame, [phases.drawStart, phases.drawEnd], [0, 100], CLAMPED)}%`,
              background: accent,
              transform: "translateY(-50%)",
              borderRadius: lineThickness / 2,
            }}
          />
        </span>
        <span
          style={{
            gridArea: "1 / 1",
            whiteSpace: "pre",
            color: emphasisColor,
            opacity: interpolate(frame, [phases.fadeStart, phases.fadeEnd], [0, 1], CLAMPED),
            transform: `translateY(${interpolate(
              frame,
              [phases.fadeStart, phases.fadeEnd],
              [8, 0],
              CLAMPED,
            )}px)`,
          }}
        >
          {replaceWith ?? highlight}
        </span>
      </span>
    );
  } else {
    highlightNode = (
      <span style={{ position: "relative", display: "inline-block" }}>
        <span style={{ color: emphasisColor }}>{highlight}</span>
        <span
          aria-hidden
          style={{
            position: "absolute",
            inset: 0,
            color: "transparent",
            backgroundClip: "text",
            WebkitBackgroundClip: "text",
            backgroundImage: `linear-gradient(110deg, transparent ${50 - shineWidth / 2}%, ${shine} 50%, transparent ${50 + shineWidth / 2}%)`,
            backgroundSize: "200% 100%",
            backgroundPosition: `${interpolate(
              frame,
              [startAt, startAt + drawDuration],
              [200, -100],
              CLAMPED,
            )}% 50%`,
          }}
        >
          {highlight}
        </span>
      </span>
    );
  }

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        background: ground,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <span style={textStyle}>
        {before}
        {highlightNode}
        {after}
      </span>
    </div>
  );
}

export function TextHighlight({ theme, ...settings }: ElementSettings["text-highlight"]) {
  if (!isTheme(theme)) return <TextHighlightView {...settings} />;
  return (
    <WarmUIProvider mode={theme.mode}>
      <TextHighlightView {...settings} colors={themeColors(theme)} mode={theme.mode} />
    </WarmUIProvider>
  );
}
