import type { CSSProperties } from "react";
import { PARAGRAPH, SLATE, cell } from "#oracle/parts.tsx";

const ALIGNS: CSSProperties["textAlign"][] = ["left", "center", "right", "justify", "start", "end"];

export function Case() {
  return (
    <>
      {ALIGNS.map((textAlign, i) => (
        <div
          key={i}
          style={cell(i, { background: SLATE, textAlign, fontSize: 15, lineHeight: 1.3 })}
        >
          {PARAGRAPH}
        </div>
      ))}
      <div style={cell(6, { background: SLATE, textAlign: "center", fontSize: 15 })}>
        <span style={{ display: "inline-block", background: "#7c3aed", padding: "2px 6px" }}>
          chip
        </span>{" "}
        centred
      </div>
      <div
        style={cell(7, { background: SLATE, textAlign: "center", fontSize: 15, direction: "rtl" })}
      >
        {PARAGRAPH}
      </div>
    </>
  );
}
