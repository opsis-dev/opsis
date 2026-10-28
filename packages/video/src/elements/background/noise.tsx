import { noise3D } from "@remotion/noise";
import React from "react";
import { AbsoluteFill, random, useCurrentFrame } from "remotion";
import { DEFAULT_GRAIN, Grain } from "#video/lib/backgrounds/grain.tsx";

const BLOBS = 5;
const BLOB_DRIFT_PCT = 40;
const BLOB_RADIUS_PCT = 30;
const BLOB_RADIUS_SPREAD_PCT = 25;
const BLOB_NOISE_SPACING = 1000;
const BLOB_OPACITY = 0.55;
const BLOB_BLUR_PX = 40;

export const NoiseField: React.FC<{
  color1: string;
  color2: string;
  scale?: number;
  speed?: number;
}> = ({ color1, color2, scale = 0.004, speed = 0.004 }) => {
  const frame = useCurrentFrame();
  const blobs = Array.from({ length: BLOBS }, (_, i) => {
    const offset = i * BLOB_NOISE_SPACING * scale;
    const x = 50 + noise3D(`x${i}`, offset, 0, frame * speed) * BLOB_DRIFT_PCT;
    const y = 50 + noise3D(`y${i}`, 0, offset, frame * speed) * BLOB_DRIFT_PCT;
    const r = BLOB_RADIUS_PCT + random(`r${i}`) * BLOB_RADIUS_SPREAD_PCT;
    return `radial-gradient(circle at ${x}% ${y}%, ${color2} 0%, transparent ${r}%)`;
  });
  return (
    <AbsoluteFill style={{ background: color1 }}>
      <AbsoluteFill
        style={{
          background: blobs.join(","),
          opacity: BLOB_OPACITY,
          filter: `blur(${BLOB_BLUR_PX}px)`,
        }}
      />
      <Grain amount={DEFAULT_GRAIN} seed={7} />
    </AbsoluteFill>
  );
};
