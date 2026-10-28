import React from "react";
import { AbsoluteFill } from "remotion";
import { DEFAULT_GRAIN, Grain } from "#video/lib/backgrounds/grain.tsx";

export const Gradient: React.FC<{ from: string; to: string; angle?: number; grain?: number }> = ({
  from,
  to,
  angle = 135,
  grain = DEFAULT_GRAIN,
}) => (
  <AbsoluteFill style={{ background: `linear-gradient(${angle}deg, ${from}, ${to})` }}>
    <Grain amount={grain} />
  </AbsoluteFill>
);
