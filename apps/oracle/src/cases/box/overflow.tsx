import type { CSSProperties } from "react";
import { COOL, LEAF, PARAGRAPH, SLATE, WARM, cell } from "#oracle/parts.tsx";

const OVERFLOWS: CSSProperties[] = [
  { overflow: "visible" },
  { overflow: "hidden" },
  { overflow: "hidden", borderRadius: 32 },
  { overflow: "hidden", borderRadius: "50%" },
  { overflowX: "hidden", overflowY: "visible" },
  { overflow: "clip" },
  { overflow: "hidden", transform: "rotate(10deg)", borderRadius: 16 },
  { overflow: "hidden", fontSize: 28 },
];

export function Case() {
  return (
    <>
      {OVERFLOWS.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, height: 100, ...style })}>
          <div
            style={{
              position: "absolute",
              left: -20,
              top: 40,
              width: 180,
              height: 30,
              background: WARM,
            }}
          />
          <div style={{ width: 60, height: 140, background: COOL, marginLeft: 50 }} />
          <div style={{ position: "absolute", left: 4, top: 4, width: 200, color: LEAF }}>
            {PARAGRAPH}
          </div>
        </div>
      ))}
    </>
  );
}
