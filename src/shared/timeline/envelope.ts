import { clamp01, lerp, sampleKeyed } from "../math";
import type { Ease, Keyframe } from "../schema/timeline";

function ease(kind: Ease, t: number): number {
  const x = clamp01(t);
  switch (kind) {
    case "in":
      return x * x;
    case "out":
      return 1 - (1 - x) * (1 - x);
    case "in-out":
      return x < 0.5 ? 2 * x * x : 1 - (-2 * x + 2) ** 2 / 2;
    case "hold":
      return 0;
    case "linear":
      return x;
  }
}

export const sortKeys = (keys: Keyframe[]): Keyframe[] =>
  keys.toSorted((a, b) => a.frame - b.frame);

const sortedEnvelopeAt = (sorted: Keyframe[], frame: number, fallback: number): number =>
  sampleKeyed(
    sorted,
    frame,
    (k) => k.frame,
    (k) => k.value,
    (a, b, t) => lerp(a.value, b.value, ease(a.ease, t)),
  ) ?? fallback;

export const envelopeAt = (keys: Keyframe[] | undefined, frame: number, fallback: number): number =>
  sortedEnvelopeAt(sortKeys(keys ?? []), frame, fallback);

const STEPS_PER_SEGMENT = 8;
const HOLD_STEP_FRAMES = 1e-3;

type EnvelopePoint = Pick<Keyframe, "frame" | "value">;

export function samplePoints(keys: Keyframe[]): EnvelopePoint[] {
  const sorted = sortKeys(keys);
  const out: EnvelopePoint[] = [];
  for (let i = 0; i < sorted.length; i++) {
    const a = sorted[i];
    out.push({ frame: a.frame, value: a.value });
    const b = sorted[i + 1];
    if (!b || b.frame <= a.frame) continue;
    if (a.ease === "hold") {
      out.push({ frame: b.frame - HOLD_STEP_FRAMES, value: a.value });
      continue;
    }
    if (a.ease === "linear") continue;
    for (let s = 1; s < STEPS_PER_SEGMENT; s++) {
      const frame = a.frame + ((b.frame - a.frame) * s) / STEPS_PER_SEGMENT;
      out.push({ frame, value: sortedEnvelopeAt(sorted, frame, a.value) });
    }
  }
  return out;
}

export const hasKeys = (env: Record<string, Keyframe[] | undefined> | undefined): boolean =>
  !!env && Object.values(env).some((k) => !!k?.length);
