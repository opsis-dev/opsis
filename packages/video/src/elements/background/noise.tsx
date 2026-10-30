import { AbsoluteFill, random, useCurrentFrame } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import { Grain } from "#video/lib/backgrounds/grain.tsx";
import { noiseField } from "#video/lib/motion/noise.ts";

const BLOB_RADIUS_SPREAD_PCT = 25;
const BLOB_NOISE_SPACING = 1000;
const Y_LANE = 731;

function seeded(key: string, seed: number): string {
  return seed === 0 ? key : `${key}-${seed}`;
}

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
  const field = noiseField(`noise:${seed}`);
  const layers = Array.from({ length: blobs }, (_, i) => {
    const offset = i * BLOB_NOISE_SPACING * scale;
    const x = 50 + field(offset, 0, frame * speed) * wander;
    const y = 50 + field(offset, Y_LANE, frame * speed) * wander;
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
