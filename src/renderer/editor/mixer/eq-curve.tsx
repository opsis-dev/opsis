import type { EqBand } from "@/shared/mix/types";

const SR = 48_000;

function coefficients(band: EqBand) {
  const A = 10 ** (band.gain / 40);
  const w0 = (2 * Math.PI * band.freq) / SR;
  const cos = Math.cos(w0);
  const alpha = Math.sin(w0) / (2 * band.q);
  if (band.type === "peak")
    return [1 + alpha * A, -2 * cos, 1 - alpha * A, 1 + alpha / A, -2 * cos, 1 - alpha / A];
  const sq = 2 * Math.sqrt(A) * alpha;
  if (band.type === "lowshelf")
    return [
      A * (A + 1 - (A - 1) * cos + sq),
      2 * A * (A - 1 - (A + 1) * cos),
      A * (A + 1 - (A - 1) * cos - sq),
      A + 1 + (A - 1) * cos + sq,
      -2 * (A - 1 + (A + 1) * cos),
      A + 1 + (A - 1) * cos - sq,
    ];
  return [
    A * (A + 1 + (A - 1) * cos + sq),
    -2 * A * (A - 1 + (A + 1) * cos),
    A * (A + 1 + (A - 1) * cos - sq),
    A + 1 - (A - 1) * cos + sq,
    2 * (A - 1 - (A + 1) * cos),
    A + 1 - (A - 1) * cos - sq,
  ];
}

export function bandResponse(band: EqBand, freq: number): number {
  const [b0, b1, b2, a0, a1, a2] = coefficients(band);
  const w = (2 * Math.PI * freq) / SR;
  const re = (x0: number, x1: number, x2: number) => x0 + x1 * Math.cos(w) + x2 * Math.cos(2 * w);
  const im = (x1: number, x2: number) => -x1 * Math.sin(w) - x2 * Math.sin(2 * w);
  const num = Math.hypot(re(b0, b1, b2), im(b1, b2));
  const den = Math.hypot(re(a0, a1, a2), im(a1, a2));
  return 20 * Math.log10(num / den);
}

export function EqCurve({
  bands,
  on,
  width,
  height,
  range = 18,
  highpass,
}: {
  bands: EqBand[];
  on: boolean;
  width: number;
  height: number;
  range?: number;
  highpass?: number | null;
}) {
  const steps = Math.max(24, Math.round(width / 2));
  const points: string[] = [];
  for (let i = 0; i <= steps; i++) {
    const f = 20 * 1000 ** (i / steps);
    let db = on
      ? bands.reduce((sum, b) => sum + (Math.abs(b.gain) < 0.05 ? 0 : bandResponse(b, f)), 0)
      : 0;
    if (highpass) db += -10 * Math.log10(1 + (highpass / f) ** 4);
    const y = height / 2 - (Math.max(-range, Math.min(range, db)) / range) * (height / 2 - 2);
    points.push(`${((i / steps) * width).toFixed(1)},${y.toFixed(1)}`);
  }
  return (
    <svg width={width} height={height} className="bg-muted/50 rounded-sm">
      <line x1={0} x2={width} y1={height / 2} y2={height / 2} className="stroke-foreground/15" />
      <polyline
        points={points.join(" ")}
        fill="none"
        strokeWidth={1.5}
        className={on || highpass ? "stroke-keyframe" : "stroke-muted-foreground/60"}
      />
    </svg>
  );
}
