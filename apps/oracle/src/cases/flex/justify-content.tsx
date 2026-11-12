import type { CSSProperties } from "react";
import { COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

const JUSTIFY: CSSProperties["justifyContent"][] = [
  "flex-start",
  "center",
  "flex-end",
  "space-between",
  "space-around",
  "space-evenly",
  "normal",
];

function Items() {
  return (
    <>
      <div style={{ width: 24, height: 30, background: WARM }} />
      <div style={{ width: 24, height: 30, background: COOL }} />
      <div style={{ width: 24, height: 30, background: LEAF }} />
    </>
  );
}

export function Case() {
  return (
    <>
      {JUSTIFY.map((justifyContent, i) => (
        <div
          key={i}
          style={cell(i, {
            background: SLATE,
            display: "flex",
            justifyContent,
            alignItems: "center",
          })}
        >
          <Items />
        </div>
      ))}
      <div
        style={cell(7, {
          background: SLATE,
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
          alignItems: "center",
        })}
      >
        <Items />
      </div>
    </>
  );
}
