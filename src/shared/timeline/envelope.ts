import type { Ease, Keyframe } from "../schema/timeline";
import { clamp } from "../math";

function ease(kind: Ease, t: number): number {
  const x = clamp(t, 0, 1);
  switch (kind) {
    case "in":
      return x * x;
    case "out":
      return 1 - (1 - x) * (1 - x);
    case "in-out":
      return x < 0.5 ? 2 * x * x : 1 - (-2 * x + 2) ** 2 / 2;
    case "hold":
      return 0;
    default:
      return x;
  }
}

export const sortKeys = (keys: Keyframe[]): Keyframe[] =>
  keys.toSorted((a, b) => a.frame - b.frame);

export function envelopeAt(keys: Keyframe[] | undefined, frame: number, fallback: number): number {
  if (!keys?.length) return fallback;
  const sorted = sortKeys(keys);
  if (frame <= sorted[0].frame) return sorted[0].value;
  for (let i = 0; i < sorted.length - 1; i++) {
    const a = sorted[i];
    const b = sorted[i + 1];
    if (frame < b.frame) {
      const span = b.frame - a.frame;
      if (span <= 0) return b.value;
      return a.value + (b.value - a.value) * ease(a.ease, (frame - a.frame) / span);
    }
  }
  return sorted[sorted.length - 1].value;
}

const STEPS_PER_SEGMENT = 8;

export function samplePoints(keys: Keyframe[]): Array<{ frame: number; value: number }> {
  const sorted = sortKeys(keys);
  const out: Array<{ frame: number; value: number }> = [];
  for (let i = 0; i < sorted.length; i++) {
    const a = sorted[i];
    out.push({ frame: a.frame, value: a.value });
    const b = sorted[i + 1];
    if (!b || b.frame <= a.frame) continue;
    if (a.ease === "hold") {
      out.push({ frame: b.frame - 1e-3, value: a.value });
      continue;
    }
    if (a.ease === "linear") continue;
    for (let s = 1; s < STEPS_PER_SEGMENT; s++) {
      const frame = a.frame + ((b.frame - a.frame) * s) / STEPS_PER_SEGMENT;
      out.push({ frame, value: envelopeAt(sorted, frame, a.value) });
    }
  }
  return out;
}

export const hasKeys = (env: Record<string, Keyframe[] | undefined> | undefined): boolean =>
  !!env && Object.values(env).some((k) => !!k?.length);
