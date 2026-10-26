import { regionBlur, type RegionBlurParams } from "@remotion/effects/region-blur";
import type { EffectDescriptor } from "remotion";
import { clamp01 } from "#video/common/math.ts";
import type { BlurArea } from "#video/generated/spec.ts";

const EASE_MS = 200;

type BlurParams = Required<RegionBlurParams> & { disabled: boolean };

function strengthAt(area: BlurArea, timeMs: number): number {
  const { fromMs, untilMs } = area;
  if (fromMs !== undefined && timeMs < fromMs) return 0;
  if (untilMs !== undefined && timeMs > untilMs) return 0;
  const easedIn = fromMs === undefined ? 1 : clamp01((timeMs - fromMs) / EASE_MS);
  const easedOut = untilMs === undefined ? 1 : clamp01((untilMs - timeMs) / EASE_MS);
  return Math.min(easedIn, easedOut);
}

export function blurParams(areas: BlurArea[], timeMs: number, fileWidth: number): BlurParams[] {
  return areas.map((area) => {
    const share = strengthAt(area, timeMs);
    return {
      topLeft: [clamp01(area.x), clamp01(area.y)],
      bottomRight: [clamp01(area.x + area.width), clamp01(area.y + area.height)],
      blurRadius: area.strength * fileWidth * share,
      feather: area.softness * fileWidth,
      roundness: area.roundness,
      disabled: share <= 0,
    };
  });
}

export const blurEffects = (
  areas: BlurArea[],
  timeMs: number,
  fileWidth: number,
): EffectDescriptor<unknown>[] => blurParams(areas, timeMs, fileWidth).map((p) => regionBlur(p));
