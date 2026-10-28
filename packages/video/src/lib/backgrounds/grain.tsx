import React from "react";
import { AbsoluteFill } from "remotion";

export const DEFAULT_GRAIN = 0.08;

export const Grain: React.FC<{ amount: number; seed?: number }> = ({ amount, seed = 3 }) =>
  amount > 0 ? (
    <AbsoluteFill style={{ opacity: amount, mixBlendMode: "overlay" }}>
      <svg width="100%" height="100%">
        <filter id={`grain-${seed}`}>
          <feTurbulence
            type="fractalNoise"
            baseFrequency="0.9"
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
