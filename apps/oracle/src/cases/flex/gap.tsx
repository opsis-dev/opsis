import type { CSSProperties } from "react";
import { COOL, LEAF, ROSE, SLATE, WARM, cell } from "#oracle/parts.tsx";

const GAPS: CSSProperties[] = [
  { display: "flex", gap: 0 },
  { display: "flex", gap: 6 },
  { display: "flex", gap: "12px 2px", flexWrap: "wrap" },
  { display: "flex", flexDirection: "column", gap: 10 },
  { display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 },
  { display: "grid", gridTemplateColumns: "1fr 1fr", rowGap: 16, columnGap: 2 },
  { display: "flex", gap: "5%" },
  { display: "flex", gap: "0.5em", fontSize: 30 },
];

function Items() {
  return (
    <>
      <div style={{ width: 24, height: 24, background: WARM }} />
      <div style={{ width: 24, height: 24, background: COOL }} />
      <div style={{ width: 24, height: 24, background: LEAF }} />
      <div style={{ width: 24, height: 24, background: ROSE }} />
    </>
  );
}

export function Case() {
  return (
    <>
      {GAPS.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, ...style })}>
          <Items />
        </div>
      ))}
    </>
  );
}
