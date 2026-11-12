import type { CSSProperties } from "react";
import { SLATE, cell } from "#oracle/parts.tsx";

const BORDERS: CSSProperties[] = [
  { border: "1px solid #f8fafc" },
  { border: "6px solid #f97316" },
  { border: "4px solid rgba(255,255,255,0.3)", borderRadius: 16 },
  { borderTop: "8px solid #0ea5e9", borderBottom: "2px solid #22c55e" },
  {
    borderWidth: "2px 10px 18px 4px",
    borderStyle: "solid",
    borderColor: "#f97316 #0ea5e9 #22c55e #f43f5e",
  },
  { border: "3px dashed #facc15", borderRadius: 12 },
  { border: "2px dotted #f8fafc" },
  { border: "12px solid #7c3aed", borderRadius: "50%" },
];

export function Case() {
  return (
    <>
      {BORDERS.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, ...style })} />
      ))}
    </>
  );
}
