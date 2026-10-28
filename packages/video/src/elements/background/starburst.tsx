import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";

const DEFAULT_RAYS = 24;
const DEFAULT_ROTATION_SPEED = 0.15;
const STARBURST_OVERSCAN = "-50%";
const STARBURST_VIGNETTE_PCT = 75;

export const Starburst: React.FC<{
  color1: string;
  color2: string;
  rays?: number;
  rotationSpeed?: number;
}> = ({ color1, color2, rays = DEFAULT_RAYS, rotationSpeed = DEFAULT_ROTATION_SPEED }) => {
  const frame = useCurrentFrame();
  const step = 360 / Math.max(2, rays);
  return (
    <AbsoluteFill style={{ overflow: "hidden", background: color1 }}>
      <div
        style={{
          position: "absolute",
          inset: STARBURST_OVERSCAN,
          background: `repeating-conic-gradient(from ${frame * rotationSpeed}deg, ${color1} 0deg ${step / 2}deg, ${color2} ${step / 2}deg ${step}deg)`,
        }}
      />
      <AbsoluteFill
        style={{
          background: `radial-gradient(circle, transparent 0%, ${color1} ${STARBURST_VIGNETTE_PCT}%)`,
        }}
      />
    </AbsoluteFill>
  );
};
