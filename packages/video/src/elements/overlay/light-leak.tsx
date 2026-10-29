import { LightLeak } from "@remotion/light-leaks";
import { AbsoluteFill } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";

export function LightLeakElement({
  seed,
  hueShift,
  strength,
  blend,
}: ElementSettings["light-leak"]) {
  return (
    <AbsoluteFill style={{ opacity: strength / 100, mixBlendMode: blend }}>
      <LightLeak seed={seed} hueShift={hueShift} />
    </AbsoluteFill>
  );
}
