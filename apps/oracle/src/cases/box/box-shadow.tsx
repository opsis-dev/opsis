import type { CSSProperties } from "react";
import { cell } from "#oracle/parts.tsx";

const SHADOWS: CSSProperties[] = [
  { boxShadow: "0 4px 12px rgba(0,0,0,0.6)" },
  { boxShadow: "0 0 0 4px #f97316" },
  { boxShadow: "0 20px 40px -10px rgba(124,58,237,0.9)", borderRadius: 20 },
  { boxShadow: "inset 0 0 20px rgba(0,0,0,0.9)" },
  {
    boxShadow: "inset 0 2px 0 rgba(255,255,255,0.4), 0 8px 24px rgba(14,165,233,0.6)",
    borderRadius: 12,
  },
  { boxShadow: "8px 8px 0 #22c55e, 16px 16px 0 #f43f5e" },
  { boxShadow: "0 0 40px 10px rgba(250,204,21,0.5)", borderRadius: "50%" },
  {
    boxShadow: "-10px -10px 30px rgba(255,255,255,0.2), 10px 10px 30px rgba(0,0,0,0.8)",
    borderRadius: 24,
  },
];

export function Case() {
  return (
    <>
      {SHADOWS.map((style, i) => (
        <div
          key={i}
          style={cell(i, {
            background: "#334155",
            inset: "auto",
            width: 100,
            height: 100,
            marginLeft: 18,
            marginTop: 24,
            ...style,
          })}
        />
      ))}
    </>
  );
}
