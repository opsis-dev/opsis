import { AbsoluteFill } from "@opsis/runtime/parts.tsx";

export function Solid({ color }: { color: string }) {
  return <AbsoluteFill style={{ background: color }} />;
}
