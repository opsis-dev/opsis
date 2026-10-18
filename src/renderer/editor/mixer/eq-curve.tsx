import { clamp } from "@/shared/math";
import { SAMPLE_RATE, type EqBand } from "@/shared/mix/types";

const MIN_HZ = 20;
const MAX_HZ = 20_000;
const RANGE_DB = 18;
const HIGHPASS_POWER = 4;

type Triple = [number, number, number];
type Biquad = { b: Triple; a: Triple };

function shelf(A: number, cos: number, sq: number, high: boolean): Biquad {
  const c = high ? -cos : cos;
  const s = high ? -1 : 1;
  return {
    b: [
      A * (A + 1 - (A - 1) * c + sq),
      s * 2 * A * (A - 1 - (A + 1) * c),
      A * (A + 1 - (A - 1) * c - sq),
    ],
    a: [A + 1 + (A - 1) * c + sq, -s * 2 * (A - 1 + (A + 1) * c), A + 1 + (A - 1) * c - sq],
  };
}

function coefficients(band: EqBand): Biquad {
  const A = 10 ** (band.gain / 40);
  const w0 = (2 * Math.PI * band.freq) / SAMPLE_RATE;
  const cos = Math.cos(w0);
  const alpha = Math.sin(w0) / (2 * band.q);
  if (band.type === "peak")
    return {
      b: [1 + alpha * A, -2 * cos, 1 - alpha * A],
      a: [1 + alpha / A, -2 * cos, 1 - alpha / A],
    };
  return shelf(A, cos, 2 * Math.sqrt(A) * alpha, band.type === "highshelf");
}

function magnitude([x0, x1, x2]: Triple, w: number): number {
  const re = x0 + x1 * Math.cos(w) + x2 * Math.cos(2 * w);
  const im = -x1 * Math.sin(w) - x2 * Math.sin(2 * w);
  return Math.hypot(re, im);
}

function bandResponse(band: EqBand, freq: number): number {
  const { b, a } = coefficients(band);
  const w = (2 * Math.PI * freq) / SAMPLE_RATE;
  return 20 * Math.log10(magnitude(b, w) / magnitude(a, w));
}

const highpassResponse = (cutoff: number, freq: number) =>
  -10 * Math.log10(1 + (cutoff / freq) ** HIGHPASS_POWER);

export function EqCurve({
  bands,
  on,
  width,
  height,
  highpass,
}: {
  bands: EqBand[];
  on: boolean;
  width: number;
  height: number;
  highpass?: number | null;
}) {
  const steps = Math.max(24, Math.round(width / 2));
  const points: string[] = [];
  for (let i = 0; i <= steps; i++) {
    const f = MIN_HZ * (MAX_HZ / MIN_HZ) ** (i / steps);
    let db = on ? bands.reduce((sum, b) => sum + bandResponse(b, f), 0) : 0;
    if (highpass) db += highpassResponse(highpass, f);
    const y = height / 2 - (clamp(db, -RANGE_DB, RANGE_DB) / RANGE_DB) * (height / 2 - 2);
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
