import { colorCorrection } from "@remotion/effects/color-correction";
import { noise } from "@remotion/effects/noise";
import { vignette } from "@remotion/effects/vignette";
import type { EffectDescriptor } from "remotion";
import type { PictureLook } from "../../shared/compile/recording";

const GRAIN_AMOUNT = 0.3;
const BRIGHTNESS_STOPS = 1;

export function lookEffects(
  look: PictureLook | undefined,
  frame: number,
): EffectDescriptor<unknown>[] {
  if (!look) return [];
  const { brightness = 0, contrast = 1, saturation = 1, warmth = 0, vignette: edges = 0 } = look;
  const grain = look.grain ?? 0;
  const colour = brightness !== 0 || contrast !== 1 || saturation !== 1 || warmth !== 0;
  return [
    ...(colour
      ? [
          colorCorrection({
            exposure: brightness * BRIGHTNESS_STOPS,
            contrast,
            saturation,
            temperature: warmth,
          }),
        ]
      : []),
    ...(edges > 0 ? [vignette({ amount: edges })] : []),
    ...(grain > 0 ? [noise({ amount: grain * GRAIN_AMOUNT, seed: frame })] : []),
  ];
}
