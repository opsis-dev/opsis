import type { ElementSettings } from "#video/generated/spec.ts";
import { SwirlField } from "#video/parts/shader-fields.tsx";

export function ShaderSwirl(settings: ElementSettings["shader-swirl"]) {
  return <SwirlField {...settings} />;
}
