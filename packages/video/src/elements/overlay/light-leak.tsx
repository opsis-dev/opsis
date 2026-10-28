import { LightLeak } from "@remotion/light-leaks";
import React from "react";

export const LightLeakElement: React.FC<{ seed: number; hueShift: number }> = (props) => (
  <LightLeak {...props} />
);
