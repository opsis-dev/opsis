import type { CSSProperties } from "react";
import { COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

const PLACES: CSSProperties[] = [
  { display: "grid", placeItems: "center" },
  { display: "grid", placeItems: "start" },
  { display: "grid", placeItems: "end" },
  { display: "grid", placeItems: "center start" },
  { display: "grid", placeItems: "stretch" },
  { display: "grid", placeContent: "center" },
  { display: "flex", placeItems: "center", justifyContent: "center" },
  { display: "grid", gridTemplateColumns: "1fr 1fr", placeItems: "center end" },
];

export function Case() {
  return (
    <>
      {PLACES.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, ...style })}>
          <div style={{ minWidth: 40, minHeight: 30, background: WARM }} />
          <div style={{ minWidth: 20, minHeight: 20, background: COOL }} />
          <div style={{ width: 30, height: 10, background: LEAF }} />
        </div>
      ))}
    </>
  );
}
