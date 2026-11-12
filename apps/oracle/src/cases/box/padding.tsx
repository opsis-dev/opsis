import type { CSSProperties } from "react";
import { SLATE, WARM, cell } from "#oracle/parts.tsx";

const PADDINGS: CSSProperties[] = [
  { padding: 0 },
  { padding: 12 },
  { padding: "4px 24px" },
  { padding: "2px 8px 30px 40px" },
  { padding: "10%" },
  { paddingLeft: 50, paddingTop: 20 },
  { padding: "0.75em", fontSize: 24 },
  { padding: 16, margin: 0, boxSizing: "content-box", width: 104, height: 116 },
];

export function Case() {
  return (
    <>
      {PADDINGS.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, ...style })}>
          <div style={{ background: WARM, height: "100%", minHeight: 20 }}>Aa</div>
        </div>
      ))}
    </>
  );
}
