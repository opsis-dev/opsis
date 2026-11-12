import type { CSSProperties } from "react";

const TRANSFORMS: CSSProperties["textTransform"][] = [
  "none",
  "uppercase",
  "lowercase",
  "capitalize",
];

export function Case() {
  return (
    <div
      style={{ position: "absolute", left: 24, top: 24, right: 24, fontSize: 28, lineHeight: 1.5 }}
    >
      {TRANSFORMS.map((textTransform) => (
        <div key={textTransform} style={{ textTransform }}>
          straße and ÉCOLE start here
        </div>
      ))}
      <div
        style={{
          textTransform: "uppercase",
          letterSpacing: "0.12em",
          fontSize: 14,
          fontWeight: 600,
        }}
      >
        small caps label
      </div>
    </div>
  );
}
