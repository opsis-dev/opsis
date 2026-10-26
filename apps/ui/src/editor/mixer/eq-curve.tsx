import { clamp } from "@opsis/video/common/math";
import type { EqBand } from "@/generated/bindings";
import { mixerBook } from "@/store/catalog";

const MIN_HZ = 20;
const MAX_HZ = 20_000;
const RANGE_DB = 18;
const HIGHPASS_POWER = 4;

type Triple = [number, number, number];
type Biquad = { b: Triple; a: Triple };

type ShelfTerms = { amp: number; cosW0: number; twoSqrtAmpAlpha: number };

function shelf({ amp, cosW0, twoSqrtAmpAlpha }: ShelfTerms, side: "low" | "high"): Biquad {
  const cos = side === "high" ? -cosW0 : cosW0;
  const sign = side === "high" ? -1 : 1;
  return {
    b: [
      amp * (amp + 1 - (amp - 1) * cos + twoSqrtAmpAlpha),
      sign * 2 * amp * (amp - 1 - (amp + 1) * cos),
      amp * (amp + 1 - (amp - 1) * cos - twoSqrtAmpAlpha),
    ],
    a: [
      amp + 1 + (amp - 1) * cos + twoSqrtAmpAlpha,
      -sign * 2 * (amp - 1 + (amp + 1) * cos),
      amp + 1 + (amp - 1) * cos - twoSqrtAmpAlpha,
    ],
  };
}

function coefficients(band: EqBand): Biquad {
  const amp = 10 ** (band.gain / 40);
  const w0 = (2 * Math.PI * band.freq) / mixerBook().sampleRate;
  const cosW0 = Math.cos(w0);
  const alpha = Math.sin(w0) / (2 * band.q);
  if (band.type === "peak")
    return {
      b: [1 + alpha * amp, -2 * cosW0, 1 - alpha * amp],
      a: [1 + alpha / amp, -2 * cosW0, 1 - alpha / amp],
    };
  const terms = { amp, cosW0, twoSqrtAmpAlpha: 2 * Math.sqrt(amp) * alpha };
  return shelf(terms, band.type === "highshelf" ? "high" : "low");
}

function magnitude([x0, x1, x2]: Triple, w: number): number {
  const re = x0 + x1 * Math.cos(w) + x2 * Math.cos(2 * w);
  const im = -x1 * Math.sin(w) - x2 * Math.sin(2 * w);
  return Math.hypot(re, im);
}

function biquadResponse({ b, a }: Biquad, freq: number): number {
  const w = (2 * Math.PI * freq) / mixerBook().sampleRate;
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
  highpass: number | null;
}) {
  const steps = Math.max(24, Math.round(width / 2));
  const biquads = on ? bands.map(coefficients) : [];
  const points: string[] = [];
  for (let i = 0; i <= steps; i++) {
    const f = MIN_HZ * (MAX_HZ / MIN_HZ) ** (i / steps);
    let db = biquads.reduce((sum, q) => sum + biquadResponse(q, f), 0);
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
