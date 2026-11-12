import type { CSSProperties } from "react";
import { cell } from "#oracle/parts.tsx";

const MODES: CSSProperties["mixBlendMode"][] = [
  "normal",
  "screen",
  "overlay",
  "lighten",
  "multiply",
  "darken",
  "plus-lighter",
  "difference",
];

export function Case() {
  return (
    <>
      {MODES.map((mixBlendMode, i) => (
        <div
          key={i}
          style={cell(i, {
            background: "linear-gradient(90deg, #0f172a, #f97316 50%, #f8fafc)",
            isolation: "isolate",
          })}
        >
          <div
            style={{
              position: "absolute",
              inset: 20,
              borderRadius: "50%",
              background: "radial-gradient(circle, #0ea5e9, #7c3aed 70%, transparent 71%)",
              mixBlendMode,
            }}
          />
        </div>
      ))}
    </>
  );
}
