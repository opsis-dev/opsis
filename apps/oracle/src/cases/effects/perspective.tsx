import type { CSSProperties } from "react";
import { SLATE, cell } from "#oracle/parts.tsx";

const SCENES: [CSSProperties, CSSProperties][] = [
  [{ perspective: 400 }, { transform: "rotateY(35deg)" }],
  [{ perspective: 200 }, { transform: "rotateX(45deg)" }],
  [{ perspective: 800 }, { transform: "rotateY(-60deg) rotateX(10deg)" }],
  [{ perspective: 300, perspectiveOrigin: "0 0" }, { transform: "rotateY(30deg)" }],
  [{}, { transform: "perspective(300px) rotateY(45deg) translateZ(-40px)" }],
  [{ perspective: 500 }, { transform: "translateZ(60px)" }],
  [
    { perspective: 400, transformStyle: "preserve-3d" },
    { transform: "rotateX(60deg) rotateZ(30deg)" },
  ],
  [{ perspective: 250 }, { transform: "rotate3d(1, 1, 0, 50deg)", backfaceVisibility: "hidden" }],
];

export function Case() {
  return (
    <>
      {SCENES.map(([stage, card], i) => (
        <div
          key={i}
          style={cell(i, { background: SLATE, display: "grid", placeItems: "center", ...stage })}
        >
          <div
            style={{
              width: 80,
              height: 80,
              borderRadius: 8,
              background: "linear-gradient(135deg, #0ea5e9, #7c3aed)",
              border: "2px solid #f8fafc",
              display: "grid",
              placeItems: "center",
              fontWeight: 700,
              ...card,
            }}
          >
            3D
          </div>
        </div>
      ))}
    </>
  );
}
