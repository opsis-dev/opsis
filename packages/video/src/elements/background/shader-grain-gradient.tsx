import type { ElementSettings } from "#video/generated/spec.ts";
import { GrainGradientField } from "#video/parts/shader-fields.tsx";

export function ShaderGrainGradient(settings: ElementSettings["shader-grain-gradient"]) {
  return <GrainGradientField {...settings} />;
}
