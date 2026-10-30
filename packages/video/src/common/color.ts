import { converter, formatCss, formatHex, formatHex8, formatHsl, formatRgb, parse } from "culori";
import { clamp, roundTo } from "#video/common/math.ts";

export type Hsva = { h: number; s: number; v: number; a: number };

export const COLOR_FORMATS = ["hex", "rgb", "hsl", "oklch"] as const;

export type ColorFormat = (typeof COLOR_FORMATS)[number];

const toHsv = converter("hsv");
const toOklch = converter("oklch");

export function parseColor(value: string): Hsva | null {
  const text = value.trim();
  if (!text || text.startsWith("var(")) return null;
  const parsed = parse(text);
  if (!parsed) return null;
  const hsv = toHsv(parsed);
  return {
    h: Number.isFinite(hsv.h) ? (hsv.h ?? 0) : 0,
    s: hsv.s,
    v: hsv.v,
    a: hsv.alpha ?? 1,
  };
}

export function formatOf(value: string): ColorFormat {
  const text = value.trim().toLowerCase();
  if (text.startsWith("rgb")) return "rgb";
  if (text.startsWith("hsl")) return "hsl";
  if (text.startsWith("oklch")) return "oklch";
  return "hex";
}

export function formatColor(color: Hsva, format: ColorFormat): string {
  const hsv = { mode: "hsv" as const, h: color.h, s: color.s, v: color.v, alpha: color.a };
  const opaque = color.a >= 1;
  switch (format) {
    case "hex":
      return (opaque ? formatHex(hsv) : formatHex8(hsv)) ?? "#000000";
    case "rgb":
      return formatRgb(opaque ? { ...hsv, alpha: undefined } : hsv) ?? "rgb(0, 0, 0)";
    case "hsl":
      return formatHsl(opaque ? { ...hsv, alpha: undefined } : hsv) ?? "hsl(0, 0%, 0%)";
    case "oklch": {
      const ok = toOklch(hsv);
      return formatCss({
        mode: "oklch",
        l: roundTo(ok.l, 3),
        c: roundTo(ok.c, 3),
        h: roundTo(Number.isFinite(ok.h) ? (ok.h ?? 0) : 0, 1),
        ...(opaque ? {} : { alpha: roundTo(color.a, 2) }),
      });
    }
  }
}

export function hueColor(h: number): string {
  return formatColor({ h, s: 1, v: 1, a: 1 }, "hex");
}

export function opaqueHex(color: Hsva): string {
  return formatColor({ ...color, a: 1 }, "hex");
}

export function colorKey(color: string): string {
  const parsed = parseColor(color);
  return parsed ? formatColor(parsed, "hex") : color.trim();
}

export function sameColor(a: string, b: string): boolean {
  return colorKey(a) === colorKey(b);
}

export function withChannel(color: Hsva, patch: Partial<Hsva>): Hsva {
  return {
    h: clamp(patch.h ?? color.h, 0, 360),
    s: clamp(patch.s ?? color.s, 0, 1),
    v: clamp(patch.v ?? color.v, 0, 1),
    a: clamp(roundTo(patch.a ?? color.a, 2), 0, 1),
  };
}
