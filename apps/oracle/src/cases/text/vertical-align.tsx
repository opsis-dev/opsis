import type { CSSProperties } from "react";
import { COOL, SLATE, WARM, cell } from "#oracle/parts.tsx";

const ALIGNS: [CSSProperties["verticalAlign"], CSSProperties["verticalAlign"]][] = [
  ["baseline", "baseline"],
  ["sub", "sub"],
  ["super", "super"],
  ["text-top", "text-top"],
  ["text-bottom", "text-bottom"],
  ["middle", "middle"],
  ["top", "4px"],
  ["bottom", "-30%"],
];

export function Case() {
  return (
    <>
      {ALIGNS.map(([chip, run], i) => (
        <div key={i} style={cell(i, { background: SLATE, fontSize: 16 })}>
          Ag{" "}
          <span
            style={{
              display: "inline-block",
              width: 20,
              height: 40,
              background: WARM,
              verticalAlign: chip,
            }}
          />{" "}
          text <span style={{ fontSize: 11, background: COOL, verticalAlign: run }}>run</span> end
        </div>
      ))}
    </>
  );
}
