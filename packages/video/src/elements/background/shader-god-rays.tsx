import type { ElementSettings } from "#video/generated/spec.ts";
import { GodRaysField } from "#video/parts/shader-fields.tsx";

export function ShaderGodRays(settings: ElementSettings["shader-god-rays"]) {
  return <GodRaysField {...settings} />;
}
