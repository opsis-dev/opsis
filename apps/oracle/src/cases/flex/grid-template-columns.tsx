import type { CSSProperties } from "react";
import { COOL, LEAF, ROSE, SLATE, WARM, cell } from "#oracle/parts.tsx";

const GRIDS: CSSProperties[] = [
  { gridTemplateColumns: "1fr 1fr 1fr" },
  { gridTemplateColumns: "40px 1fr" },
  { gridTemplateColumns: "repeat(3, minmax(0, 1fr))", gap: 4 },
  { gridTemplateColumns: "auto 1fr auto", gap: 4 },
  { gridTemplateColumns: "2fr 1fr", gridAutoRows: 30, gap: 2 },
  { gridTemplateColumns: "repeat(auto-fill, 30px)", gap: 4 },
  { gridTemplateColumns: "1fr 1fr", gridTemplateRows: "1fr 2fr" },
  { gridTemplateColumns: "25% 50% 25%", alignItems: "end" },
];

const COLORS = [WARM, COOL, LEAF, ROSE, WARM, COOL];

export function Case() {
  return (
    <>
      {GRIDS.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, display: "grid", ...style })}>
          {COLORS.map((background, j) => (
            <div key={j} style={{ minHeight: 20, minWidth: 10, background, fontSize: 10 }}>
              {j === 1 ? "auto" : ""}
            </div>
          ))}
        </div>
      ))}
    </>
  );
}
