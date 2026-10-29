import type { ElementSettings } from "#video/generated/spec.ts";
import { DitheringField } from "#video/parts/shader-fields.tsx";

export function ShaderDithering(settings: ElementSettings["shader-dithering"]) {
  return <DitheringField {...settings} />;
}
