import { AbsoluteFill } from "@opsis/runtime/parts.tsx";

const GRAIN_FREQUENCY = 0.9;

export function Grain({ amount, size, seed = 3 }: { amount: number; size: number; seed?: number }) {
  return amount > 0 ? (
    <AbsoluteFill style={{ opacity: amount, mixBlendMode: "overlay" }}>
      <svg width="100%" height="100%">
        <filter id={`grain-${seed}`}>
          <feTurbulence
            type="fractalNoise"
            baseFrequency={GRAIN_FREQUENCY / size}
            numOctaves={2}
            seed={seed}
            stitchTiles="stitch"
          />
          <feColorMatrix type="saturate" values="0" />
        </filter>
        <rect width="100%" height="100%" filter={`url(#grain-${seed})`} />
      </svg>
    </AbsoluteFill>
  ) : null;
}
