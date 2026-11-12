import type { CSSProperties } from "react";
import { SLATE, cell } from "#oracle/parts.tsx";

const SCALES: CSSProperties[] = [
  { scale: 1 },
  { scale: 0.5 },
  { scale: 1.4 },
  { scale: "1.5 0.6" },
  { scale: "-1 1" },
  { scale: 0.001 },
  { transform: "scale(0.7, 1.2)" },
  { scale: 0.8, transformOrigin: "0 0" },
];

export function Case() {
  return (
    <>
      {SCALES.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, display: "grid", placeItems: "center" })}>
          <div
            style={{
              width: 70,
              height: 70,
              background: "linear-gradient(90deg, #f97316, #7c3aed)",
              borderRadius: 12,
              fontWeight: 700,
              display: "grid",
              placeItems: "center",
              ...style,
            }}
          >
            Ab
          </div>
        </div>
      ))}
    </>
  );
}
