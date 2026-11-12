import type { CSSProperties } from "react";
import { COOL, LEAF, ROSE, SLATE, WARM, cell } from "#oracle/parts.tsx";

const WRAPS: CSSProperties[] = [
  { flexWrap: "nowrap" },
  { flexWrap: "wrap" },
  { flexWrap: "wrap-reverse" },
  { flexWrap: "wrap", gap: 6 },
  { flexWrap: "wrap", justifyContent: "center", gap: 4 },
  { flexWrap: "wrap", alignContent: "space-between" },
  { flexWrap: "wrap", flexDirection: "column", height: 100 },
  { flexWrap: "wrap", alignContent: "center", alignItems: "flex-end" },
];

const COLORS = [WARM, COOL, LEAF, ROSE, WARM, COOL, LEAF];

export function Case() {
  return (
    <>
      {WRAPS.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, display: "flex", ...style })}>
          {COLORS.map((background, j) => (
            <div
              key={j}
              style={{ width: 30 + (j % 3) * 6, height: 20 + (j % 2) * 10, background }}
            />
          ))}
        </div>
      ))}
    </>
  );
}
