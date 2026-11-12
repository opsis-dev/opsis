import type { CSSProperties } from "react";
import { COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

const DIRECTIONS: CSSProperties["flexDirection"][] = [
  "row",
  "column",
  "row-reverse",
  "column-reverse",
];

function Items() {
  return (
    <>
      <div style={{ width: 30, height: 30, background: WARM }} />
      <div style={{ width: 40, height: 20, background: COOL }} />
      <div style={{ width: 20, height: 40, background: LEAF }} />
    </>
  );
}

export function Case() {
  return (
    <>
      {DIRECTIONS.map((flexDirection, i) => (
        <div key={i} style={cell(i, { background: SLATE, display: "flex", flexDirection })}>
          <Items />
        </div>
      ))}
      {DIRECTIONS.map((flexDirection, i) => (
        <div
          key={`gap-${i}`}
          style={cell(i + 4, {
            background: SLATE,
            display: "flex",
            flexDirection,
            gap: 8,
            padding: 8,
            alignItems: "center",
          })}
        >
          <Items />
        </div>
      ))}
    </>
  );
}
