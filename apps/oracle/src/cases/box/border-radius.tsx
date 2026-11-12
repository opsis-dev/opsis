import type { CSSProperties } from "react";
import { cell } from "#oracle/parts.tsx";

const RADII: CSSProperties[] = [
  { borderRadius: 8 },
  { borderRadius: 24 },
  { borderRadius: "50%" },
  { borderRadius: 9999 },
  { borderRadius: "40px 0" },
  { borderRadius: "10px 20px 30px 40px" },
  { borderRadius: "50% / 20%" },
  { borderTopLeftRadius: 60, borderBottomRightRadius: 12 },
];

export function Case() {
  return (
    <>
      {RADII.map((style, i) => (
        <div
          key={i}
          style={cell(i, {
            background: "linear-gradient(135deg, #f97316, #7c3aed)",
            height: i === 3 ? 60 : 148,
            ...style,
          })}
        />
      ))}
    </>
  );
}
