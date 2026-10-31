import { interpolate } from "@opsis/runtime/interpolate.ts";
import {
  clampChroma,
  converter,
  formatRgb,
  interpolate as colorRamp,
  type Oklch,
  parse,
  type Rgb,
} from "culori";

const toRgb = converter("rgb");
const toOklch = converter("oklch");

const BLACK: Rgb = { mode: "rgb", r: 0, g: 0, b: 0, alpha: 1 };

function isDev(): boolean {
  const env = (globalThis as { process?: { env?: { NODE_ENV?: string } } }).process?.env;
  return env?.NODE_ENV !== "production";
}

export function parseColor(c: string): Rgb {
  const s = c.trim();

  if (s.startsWith("var(")) {
    if (isDev()) {
      console.warn(
        `[ui-kit] parseColor cannot resolve CSS variable "${s}" in the film's per-frame render. ` +
          "Animated colors must be concrete oklch/hex/rgb values supplied via the theme. " +
          "Falling back to the JS default.",
      );
    }
    return { ...BLACK };
  }

  const rgb = toRgb(parse(s));
  if (!rgb) {
    if (isDev()) {
      console.warn(`[ui-kit] parseColor could not parse "${s}"; using black.`);
    }
    return { ...BLACK };
  }

  return { mode: "rgb", r: rgb.r, g: rgb.g, b: rgb.b, alpha: rgb.alpha ?? 1 };
}

export function oklchToRgb(l: number, c: number, h: number): Rgb {
  const mapped = clampChroma({ mode: "oklch", l, c, h }, "oklch", "rgb");
  return toRgb(mapped);
}

export function rgbToOklch(rgb: Rgb): Oklch {
  const { l, c, h } = toOklch(rgb);
  return { mode: "oklch", l, c, h: Number.isFinite(h) ? h : 0 };
}

function resolveColorString(s: string): string {
  const trimmed = s.trim();
  if (trimmed.startsWith("var(")) {
    if (isDev()) {
      console.warn(
        `[ui-kit] mixOklch cannot resolve CSS variable "${trimmed}" in the film's per-frame render. ` +
          "Animated colors must be concrete oklch/hex/rgb values supplied via the theme. " +
          "Falling back to the JS default.",
      );
    }
    return "#000";
  }
  if (!parse(trimmed)) {
    if (isDev()) {
      console.warn(`[ui-kit] mixOklch could not parse "${trimmed}"; using black.`);
    }
    return "#000";
  }
  return trimmed;
}

export function mixOklch(a: string, b: string, t: number): string {
  const mixed = clampChroma(
    colorRamp([resolveColorString(a), resolveColorString(b)], "oklch")(t),
    "oklch",
    "rgb",
  );
  return toCss(toRgb(mixed));
}

export function interpolateColors(
  input: number,
  inputRange: readonly number[],
  colors: readonly string[],
): string {
  const position = interpolate(
    input,
    inputRange,
    inputRange.map((_, i) => i),
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  const index = Math.max(0, Math.min(colors.length - 2, Math.floor(position)));
  const from = colors[index] ?? "#000";
  return mixOklch(from, colors[index + 1] ?? from, position - index);
}

export function toCss(color: Rgb): string {
  return formatRgb(color);
}

export function withAlpha(color: string, alpha: number): string {
  const { r, g, b } = parseColor(color);
  return formatRgb({ mode: "rgb", r, g, b, alpha });
}
