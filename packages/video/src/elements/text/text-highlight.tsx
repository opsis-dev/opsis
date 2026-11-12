import { interpolate } from "@opsis/runtime/interpolate.ts";
import { spring } from "@opsis/runtime/spring.ts";
import type React from "react";
import { MAIN_FONT } from "#video/common/fonts.ts";
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
  swapLength: number,
): { drawStart: number; drawEnd: number; fadeStart: number; fadeEnd: number } {
  const drawEnd = startAt + drawDuration;
  return {
    drawStart: startAt,
    drawEnd,
    fadeStart: drawEnd,
    fadeEnd: drawEnd + swapLength,
  };
}

function shinePosition(frame: number, startAt: number, drawDuration: number, sweeps: number) {
  const count = Math.max(1, sweeps);
  const progress = interpolate(
    frame,
    [startAt, startAt + drawDuration * count],
    [0, count],
    CLAMPED,
  );
  const pass = progress >= count ? 1 : progress - Math.floor(progress);
  return interpolate(pass, [0, 1], [200, -100], CLAMPED);
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
  swapLength,
  thickness,
  shineColor,
  fontSize,
  fontWeight,
  speed,
  x,
  y,
  letterSpacing,
  lineHeight,
  textWidth,
  markerOpacity,
  markerHeight,
  shineWidth,
  sweeps,
  bandAngle,
}: Omit<ElementSettings["text-highlight"], "theme"> & {
  colors?: Partial<WarmTheme>;
  mode?: "light" | "dark";
}) {
  const frame = useAuthoredFrame() * speed;
  const { fps } = useAuthoredConfig();
  const t = useWarmTheme(colors, mode);
  const ground = background ?? t.background;
  const face = resolveFont(fontFamily || t.fontFamily) ?? MAIN_FONT;
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

  let highlightNode: React.ReactNode;

  if (preset === "marker") {
    highlightNode = (
      <span style={{ position: "relative", display: "inline-block" }}>
        <span
          aria-hidden
          style={{
            position: "absolute",
            left: "-0.12em",
            right: "-0.12em",
            bottom: "-0.02em",
            height: `${markerHeight * 100}%`,
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
    const phases = strikethroughPhases(startAt, drawDuration, swapLength);
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
            backgroundImage: `linear-gradient(${bandAngle}deg, transparent ${50 - shineWidth / 2}%, ${shine} 50%, transparent ${50 + shineWidth / 2}%)`,
            backgroundSize: "200% 100%",
            backgroundPosition: `${shinePosition(frame, startAt, drawDuration, sweeps)}% 50%`,
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
      }}
    >
      <span
        style={{
          ...textStyle,
          position: "absolute",
          left: `${x * 100}%`,
          top: `${y * 100}%`,
          width: `${textWidth}%`,
          transform: "translate(-50%, -50%)",
        }}
      >
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
