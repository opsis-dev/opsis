import type { ElementSettings } from "#video/generated/spec.ts";
import { WarpField } from "#video/parts/shader-fields.tsx";

export function ShaderWarp(settings: ElementSettings["shader-warp"]) {
  return <WarpField {...settings} />;
}
