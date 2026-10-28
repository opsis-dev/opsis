export type ChartDatum = {
  label: string;
  value: number;
  color?: string;
  delta?: string;
};

export type ChartPoint = {
  x: number;
  y: number;
  label?: string;
};

export type ChartDomain = {
  min: number;
  max: number;
  span: number;
};

export type PlotArea = {
  left: number;
  top: number;
  right: number;
  bottom: number;
  width: number;
  height: number;
};

export type PlotInsets = {
  top?: number;
  right?: number;
  bottom?: number;
  left?: number;
};

export type PlottedPoint = {
  x: number;
  y: number;
  value: number;
  label?: string;
};

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

export function getChartDomain(
  values: number[],
  { includeZero = false }: { includeZero?: boolean } = {},
): ChartDomain {
  const finite = values.filter((value) => Number.isFinite(value));

  if (finite.length === 0) {
    return { min: 0, max: 1, span: 1 };
  }

  const min = Math.min(...finite, includeZero ? 0 : Infinity);
  const max = Math.max(...finite, includeZero ? 0 : -Infinity);

  if (min === max) {
    const padding = Math.abs(max) || 1;
    return { min: min - padding, max: max + padding, span: padding * 2 };
  }

  return { min, max, span: max - min };
}

function niceStep(rawStep: number): number {
  if (rawStep <= 0) return 1;
  const magnitude = 10 ** Math.floor(Math.log10(rawStep));
  const normalized = rawStep / magnitude;
  const step = normalized <= 1 ? 1 : normalized <= 2 ? 2 : normalized <= 5 ? 5 : 10;
  return step * magnitude;
}

export function niceDomain(
  values: number[],
  { includeZero = true, tickCount = 4 }: { includeZero?: boolean; tickCount?: number } = {},
): ChartDomain & { ticks: number[] } {
  const raw = getChartDomain(values, { includeZero });
  const step = niceStep(raw.span / Math.max(1, tickCount));
  const min = Math.floor(raw.min / step) * step;
  const max = Math.ceil(raw.max / step) * step;
  const ticks: number[] = [];

  for (let index = 0; min + index * step <= max + step * 1e-6; index += 1) {
    ticks.push(Number((min + index * step).toPrecision(12)));
  }

  return { min, max, span: max - min || 1, ticks };
}

export function scaleValue(value: number, min: number, max: number): number {
  if (max === min) return 0;
  return (value - min) / (max - min);
}

export function getPlotArea(width: number, height: number, insets: PlotInsets = {}): PlotArea {
  const top = insets.top ?? 24;
  const right = insets.right ?? 24;
  const bottom = insets.bottom ?? 24;
  const left = insets.left ?? 24;

  return {
    left,
    top,
    right: width - right,
    bottom: height - bottom,
    width: Math.max(1, width - left - right),
    height: Math.max(1, height - top - bottom),
  };
}

export function plotPoints(
  points: ChartPoint[],
  plot: PlotArea,
  yDomain: ChartDomain,
): PlottedPoint[] {
  const xDomain = getChartDomain(points.map((point) => point.x));

  return points.map((point) => ({
    x: plot.left + scaleValue(point.x, xDomain.min, xDomain.max) * plot.width,
    y: plot.bottom - clamp01(scaleValue(point.y, yDomain.min, yDomain.max)) * plot.height,
    value: point.y,
    label: point.label,
  }));
}

export function buildLinePath(points: { x: number; y: number }[]): string {
  if (points.length === 0) return "";
  return points
    .map((point, index) => `${index === 0 ? "M" : "L"} ${round(point.x)} ${round(point.y)}`)
    .join(" ");
}

export function buildSmoothPath(points: { x: number; y: number }[], tension = 0.42): string {
  if (points.length < 3) return buildLinePath(points);

  const parts = [`M ${round(points[0].x)} ${round(points[0].y)}`];

  for (let index = 0; index < points.length - 1; index += 1) {
    const previous = points[index - 1] ?? points[index];
    const current = points[index];
    const next = points[index + 1];
    const after = points[index + 2] ?? next;

    const lowerBound = Math.min(current.y, next.y);
    const upperBound = Math.max(current.y, next.y);
    const clampY = (value: number) => Math.min(upperBound, Math.max(lowerBound, value));

    const c1x = current.x + ((next.x - previous.x) / 6) * tension * 2;
    const c1y = clampY(current.y + ((next.y - previous.y) / 6) * tension * 2);
    const c2x = next.x - ((after.x - current.x) / 6) * tension * 2;
    const c2y = clampY(next.y - ((after.y - current.y) / 6) * tension * 2);

    parts.push(
      `C ${round(c1x)} ${round(c1y)}, ${round(c2x)} ${round(c2y)}, ${round(next.x)} ${round(next.y)}`,
    );
  }

  return parts.join(" ");
}

export function buildAreaPath(
  linePath: string,
  points: { x: number; y: number }[],
  baselineY: number,
): string {
  if (points.length === 0 || linePath === "") return "";
  const first = points[0];
  const last = points[points.length - 1];
  return `${linePath} L ${round(last.x)} ${round(baselineY)} L ${round(first.x)} ${round(baselineY)} Z`;
}

export function formatCompactNumber(value: number, maximumFractionDigits = 1): string {
  return new Intl.NumberFormat("en", {
    notation: "compact",
    maximumFractionDigits,
  }).format(value);
}

export function formatAxisValue(value: number): string {
  return formatCompactNumber(value, Math.abs(value) < 10 ? 1 : 0);
}

export function readDelta(delta: string | undefined): {
  direction: "up" | "down" | "flat";
  text: string;
} {
  if (!delta) return { direction: "flat", text: "" };
  const trimmed = delta.trim();
  if (trimmed.startsWith("+")) return { direction: "up", text: trimmed };
  if (trimmed.startsWith("-") || trimmed.startsWith("−")) {
    return { direction: "down", text: trimmed };
  }
  return { direction: "flat", text: trimmed };
}

function round(value: number): number {
  return Math.round(value * 100) / 100;
}
