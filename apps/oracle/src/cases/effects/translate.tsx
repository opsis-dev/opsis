import type { CSSProperties } from "react";
import { SLATE, WARM, cell } from "#oracle/parts.tsx";

const MOVES: CSSProperties[] = [
  { translate: "0 0" },
  { translate: "20px 30px" },
  { translate: "50%" },
  { translate: "-25% 120%" },
  { translate: "10.5px 0.25px" },
  { transform: "translate(30px, -10px)" },
  { transform: "translateX(40px) translateY(60px)" },
  { translate: "20px 20px", transform: "translate(10px, 10px)" },
];

export function Case() {
  return (
    <>
      {MOVES.map((style, i) => (
        <div key={i} style={cell(i, { background: SLATE, overflow: "hidden" })}>
          <div style={{ width: 50, height: 50, background: WARM, ...style }} />
        </div>
      ))}
    </>
  );
}
