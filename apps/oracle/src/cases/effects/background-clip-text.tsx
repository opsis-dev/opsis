import type { CSSProperties } from "react";
import { cell } from "#oracle/parts.tsx";

const FILLS: CSSProperties[] = [
  { background: "linear-gradient(90deg, #f97316, #7c3aed)" },
  { background: "linear-gradient(180deg, #f8fafc 40%, #0ea5e9)" },
  { background: "radial-gradient(circle at 30% 50%, #facc15, #f43f5e 60%)" },
  { background: "repeating-linear-gradient(45deg, #22c55e 0 4px, #0ea5e9 4px 8px)" },
  {
    background: "linear-gradient(90deg, rgba(255,255,255,0.2), #fff 50%, rgba(255,255,255,0.2))",
    backgroundSize: "200% 100%",
    backgroundPosition: "30% 0",
  },
  { background: "conic-gradient(#f97316, #0ea5e9, #22c55e, #f97316)" },
  { background: "linear-gradient(in oklch 90deg, #f43f5e, #0ea5e9)" },
  {
    background: "linear-gradient(#f8fafc, #94a3b8)",
    filter: "drop-shadow(0 2px 4px rgba(0,0,0,0.8))",
  },
];

export function Case() {
  return (
    <>
      {FILLS.map((style, i) => (
        <div
          key={i}
          style={cell(i, {
            fontSize: 44,
            fontWeight: 900,
            lineHeight: 1,
            letterSpacing: "-0.03em",
            color: "transparent",
            WebkitBackgroundClip: "text",
            backgroundClip: "text",
            ...style,
          })}
        >
          Big type
        </div>
      ))}
    </>
  );
}
