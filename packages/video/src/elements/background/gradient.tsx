import { AbsoluteFill } from "remotion";
import type { ElementSettings, GradientShape } from "#video/generated/spec.ts";
import { Grain } from "#video/lib/backgrounds/grain.tsx";

function fill(shape: GradientShape, angle: number, from: string, to: string): string {
  switch (shape) {
    case "radial":
      return `radial-gradient(circle, ${from}, ${to})`;
    case "conic":
      return `conic-gradient(from ${angle}deg, ${from}, ${to}, ${from})`;
    case "linear":
      return `linear-gradient(${angle}deg, ${from}, ${to})`;
  }
}

export function Gradient({
  from,
  to,
  angle,
  grain,
  shape,
  grainSize,
}: ElementSettings["gradient"]) {
  return (
    <AbsoluteFill style={{ background: fill(shape, angle, from, to) }}>
      <Grain amount={grain} size={grainSize} />
    </AbsoluteFill>
  );
}
