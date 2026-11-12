import type { CSSProperties } from "react";
import { cell } from "#oracle/parts.tsx";

const FILLS: CSSProperties[] = [
  { backgroundColor: "#f97316" },
  { backgroundColor: "rgb(14 165 233 / 50%)" },
  { backgroundColor: "rgba(255, 255, 255, 0.08)" },
  { backgroundColor: "hsl(280 70% 50%)" },
  { background: "#22c55e" },
  { background: "transparent", outline: "1px solid #f8fafc" },
  { backgroundColor: "#ffffff" },
  { backgroundColor: "#00000080" },
];

export function Case() {
  return (
    <>
      {FILLS.map((style, i) => (
        <div key={i} style={cell(i, style)} />
      ))}
    </>
  );
}
