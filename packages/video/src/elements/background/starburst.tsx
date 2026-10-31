import { useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

const STARBURST_OVERSCAN = "-50%";

export function Starburst({
  color1,
  color2,
  rays,
  rotationSpeed,
  edgeFade,
  rayWidth,
  centerX,
}: ElementSettings["starburst"]) {
  const frame = useFrame();
  const step = 360 / Math.max(2, rays);
  const split = (step * rayWidth) / 100;
  const across = (centerX + 50) / 2;
  return (
    <AbsoluteFill style={{ overflow: "hidden", background: color1 }}>
      <div
        style={{
          position: "absolute",
          inset: STARBURST_OVERSCAN,
          background: `repeating-conic-gradient(from ${frame * rotationSpeed}deg at ${across}% 50%, ${color1} 0deg ${split}deg, ${color2} ${split}deg ${step}deg)`,
        }}
      />
      <AbsoluteFill
        style={{
          background: `radial-gradient(circle at ${centerX}% 50%, transparent 0%, ${color1} ${edgeFade}%)`,
        }}
      />
    </AbsoluteFill>
  );
}
