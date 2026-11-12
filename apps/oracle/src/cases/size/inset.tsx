import type { CSSProperties } from "react";
import { COOL, LEAF, ROSE, SLATE, WARM, cell } from "#oracle/parts.tsx";

const INSETS: CSSProperties[] = [
  { inset: 0, background: WARM },
  { inset: 20, background: COOL },
  { inset: "10px 40px", background: LEAF },
  { inset: "8px 16px 32px 48px", background: ROSE },
  { inset: "25%", background: WARM },
  { inset: "-10px", background: COOL },
  { inset: "0 auto auto 0", width: 50, height: 50, background: LEAF },
  { inset: "auto 0 0 auto", width: "50%", height: "30%", background: ROSE },
];

export function Case() {
  return (
    <>
      {INSETS.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, overflow: "hidden" })}>
          <div style={{ position: "absolute", ...style }} />
        </div>
      ))}
    </>
  );
}
