import React from "react";
import { AbsoluteFill } from "remotion";
import { Grain } from "#video/lib/backgrounds/grain.tsx";

const PAPER_GRAIN = 0.18;

export const Paper: React.FC<{ color: string; grain?: number }> = ({
  color,
  grain = PAPER_GRAIN,
}) => (
  <AbsoluteFill style={{ background: color }}>
    <Grain amount={grain} seed={11} />
  </AbsoluteFill>
);
