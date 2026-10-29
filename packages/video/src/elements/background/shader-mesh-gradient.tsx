import type { ElementSettings } from "#video/generated/spec.ts";
import { MeshGradientField } from "#video/parts/shader-fields.tsx";

export function ShaderMeshGradient(settings: ElementSettings["shader-mesh-gradient"]) {
  return <MeshGradientField {...settings} />;
}
