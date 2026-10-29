import { noise3D } from "@remotion/noise";
import { AbsoluteFill, random, useCurrentFrame } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import { Grain } from "#video/lib/backgrounds/grain.tsx";

const BLOB_RADIUS_SPREAD_PCT = 25;
const BLOB_NOISE_SPACING = 1000;

const seeded = (key: string, seed: number): string => (seed === 0 ? key : `${key}-${seed}`);

export function NoiseField({
  color1,
  color2,
  scale,
  speed,
  blobs,
  wander,
  blobSize,
  intensity,
  softness,
  seed,
  grain,
  grainSize,
}: ElementSettings["noise"]) {
  const frame = useCurrentFrame();
  const layers = Array.from({ length: blobs }, (_, i) => {
    const offset = i * BLOB_NOISE_SPACING * scale;
    const x = 50 + noise3D(seeded(`x${i}`, seed), offset, 0, frame * speed) * wander;
    const y = 50 + noise3D(seeded(`y${i}`, seed), 0, offset, frame * speed) * wander;
    const r = blobSize + random(seeded(`r${i}`, seed)) * BLOB_RADIUS_SPREAD_PCT;
    return `radial-gradient(circle at ${x}% ${y}%, ${color2} 0%, transparent ${r}%)`;
  });
  return (
    <AbsoluteFill style={{ background: color1 }}>
      <AbsoluteFill
        style={{
          background: layers.join(","),
          opacity: intensity,
          filter: `blur(${softness}px)`,
        }}
      />
      <Grain amount={grain} size={grainSize} seed={7} />
    </AbsoluteFill>
  );
}
