import type { CSSProperties } from "react";
import { COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

const ALIGNS: CSSProperties["alignItems"][] = [
  "stretch",
  "flex-start",
  "center",
  "flex-end",
  "baseline",
  "start",
  "end",
];

function Items() {
  return (
    <>
      <div style={{ width: 30, minHeight: 20, background: WARM, fontSize: 10 }}>a</div>
      <div style={{ width: 30, minHeight: 50, background: COOL, fontSize: 24 }}>b</div>
      <div style={{ width: 30, minHeight: 30, background: LEAF, fontSize: 14, paddingTop: 10 }}>
        c
      </div>
    </>
  );
}

export function Case() {
  return (
    <>
      {ALIGNS.map((alignItems, i) => (
        <div key={i} style={cell(i, { background: SLATE, display: "flex", alignItems, gap: 4 })}>
          <Items />
        </div>
      ))}
      <div
        style={cell(7, {
          background: SLATE,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 4,
        })}
      >
        <Items />
        <div style={{ alignSelf: "flex-end", width: 30, height: 10, background: WARM }} />
      </div>
    </>
  );
}
