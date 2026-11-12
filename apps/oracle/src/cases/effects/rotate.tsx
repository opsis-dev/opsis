import type { CSSProperties } from "react";
import { SLATE, cell } from "#oracle/parts.tsx";

const TURNS: CSSProperties[] = [
  { rotate: "0deg" },
  { rotate: "15deg" },
  { rotate: "45deg" },
  { rotate: "-90deg" },
  { rotate: "0.5turn" },
  { rotate: "3.3deg" },
  { transform: "rotate(30deg)" },
  { rotate: "20deg", transformOrigin: "0 0" },
];

export function Case() {
  return (
    <>
      {TURNS.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, display: "grid", placeItems: "center" })}>
          <div
            style={{
              width: 80,
              height: 40,
              background: "linear-gradient(90deg, #0ea5e9, #22c55e)",
              border: "2px solid #f8fafc",
              fontSize: 14,
              ...style,
            }}
          >
            Rotate
          </div>
        </div>
      ))}
    </>
  );
}
