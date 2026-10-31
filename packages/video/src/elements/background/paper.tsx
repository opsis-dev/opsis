import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import { Grain } from "#video/lib/backgrounds/grain.tsx";

export function Paper({ color, grain, grainSize }: ElementSettings["paper"]) {
  return (
    <AbsoluteFill style={{ background: color }}>
      <Grain amount={grain} size={grainSize} seed={11} />
    </AbsoluteFill>
  );
}
