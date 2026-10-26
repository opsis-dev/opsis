import { parseColor } from "@opsis/video/common/color";
import { percentText } from "@opsis/video/common/format";
import { clamp } from "@opsis/video/common/math";

export type GradientStop = { color: string; at: number };

export type Gradient = {
  type: "linear" | "radial";
  angle: number;
  stops: GradientStop[];
};

export const MIN_STOPS = 2;
export const MAX_STOPS = 6;

const SIDES: Partial<Record<string, number>> = {
  "to top": 0,
  "to top right": 45,
  "to right top": 45,
  "to right": 90,
  "to bottom right": 135,
  "to right bottom": 135,
  "to bottom": 180,
  "to bottom left": 225,
  "to left bottom": 225,
  "to left": 270,
  "to top left": 315,
  "to left top": 315,
};

export function splitTopLevel(text: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let current = "";
  for (const ch of text) {
    if (ch === "(") depth++;
    if (ch === ")") depth--;
    if (ch === "," && depth === 0) {
      parts.push(current.trim());
      current = "";
    } else current += ch;
  }
  if (current.trim()) parts.push(current.trim());
  return parts;
}

function parseStop(part: string): { color: string; at?: number } | null {
  const match = /^(.*?)\s+(-?[\d.]+)%$/.exec(part);
  const color = (match ? match[1] : part).trim();
  if (!parseColor(color)) return null;
  return match ? { color, at: clamp(Number(match[2]), 0, 100) } : { color };
}

function angleOf(part: string): number | null {
  const deg = /^(-?[\d.]+)deg$/.exec(part);
  if (deg) return ((Number(deg[1]) % 360) + 360) % 360;
  return SIDES[part.replace(/\s+/g, " ").toLowerCase()] ?? null;
}

function spread(stops: Array<{ color: string; at?: number }>): GradientStop[] {
  const last = Math.max(1, stops.length - 1);
  return stops.map((s, i) => ({ color: s.color, at: s.at ?? Math.round((i / last) * 100) }));
}

export function parseGradient(text: string): Gradient | null {
  const match = /^(linear|radial)-gradient\((.*)\)$/s.exec(text.trim());
  if (!match) return null;
  const type = match[1] === "radial" ? "radial" : "linear";
  const parts = splitTopLevel(match[2]);
  let angle = 180;
  const first = parts[0] ?? "";
  if (!parseStop(first)) {
    parts.shift();
    if (type === "linear") angle = angleOf(first) ?? angle;
  }
  const stops = parts.map(parseStop);
  if (stops.length < MIN_STOPS || stops.some((s) => s === null)) return null;
  return { type, angle, stops: spread(stops.flatMap((s) => s ?? [])) };
}

export const sortStops = (gradient: Gradient): Gradient => ({
  ...gradient,
  stops: gradient.stops.toSorted((a, b) => a.at - b.at),
});

export function formatGradient({ type, angle, stops }: Gradient): string {
  const list = stops.map((s) => `${s.color} ${percentText(s.at / 100)}`).join(", ");
  return type === "linear"
    ? `linear-gradient(${Math.round(angle)}deg, ${list})`
    : `radial-gradient(circle, ${list})`;
}

export function insertStop(gradient: Gradient, at: number): Gradient {
  if (gradient.stops.length >= MAX_STOPS) return gradient;
  const sorted = gradient.stops.toSorted((a, b) => a.at - b.at);
  const after = sorted.find((s) => s.at >= at) ?? sorted.at(-1);
  const color = after?.color ?? "#ffffff";
  return { ...gradient, stops: [...gradient.stops, { color, at: clamp(Math.round(at), 0, 100) }] };
}
