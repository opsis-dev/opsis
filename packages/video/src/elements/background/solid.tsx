import { AbsoluteFill } from "remotion";

export function Solid({ color }: { color: string }) {
  return <AbsoluteFill style={{ background: color }} />;
}
