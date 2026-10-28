import React from "react";
import { AbsoluteFill } from "remotion";

export const Solid: React.FC<{ color: string }> = ({ color }) => (
  <AbsoluteFill style={{ background: color }} />
);
