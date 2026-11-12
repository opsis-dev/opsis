import type { CSSProperties } from "react";

const STROKES: CSSProperties[] = [
  { WebkitTextStroke: "1px #f8fafc", color: "transparent" },
  { WebkitTextStroke: "3px #0f172a", color: "#f8fafc", paintOrder: "stroke fill" },
  { WebkitTextStroke: "2px #f97316", color: "#0ea5e9" },
  {
    WebkitTextStrokeWidth: 4,
    WebkitTextStrokeColor: "#7c3aed",
    color: "#f8fafc",
    paintOrder: "stroke",
  },
];

export function Case() {
  return (
    <div
      style={{
        position: "absolute",
        left: 24,
        top: 16,
        fontSize: 52,
        fontWeight: 900,
        lineHeight: 1.25,
      }}
    >
      {STROKES.map((style, i) => (
        <div key={i} style={style}>
          Outline
        </div>
      ))}
    </div>
  );
}
