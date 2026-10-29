import type { ElementSettings } from "#video/generated/spec.ts";
import { PerlinNoiseField } from "#video/parts/shader-fields.tsx";

export function ShaderPerlinNoise(settings: ElementSettings["shader-perlin-noise"]) {
  return <PerlinNoiseField {...settings} />;
}
