import type { CSSProperties } from "react";

const VARIATIONS: CSSProperties[] = [
  { fontFamily: "Fraunces", fontVariationSettings: '"opsz" 9' },
  { fontFamily: "Fraunces", fontVariationSettings: '"opsz" 144' },
  { fontFamily: "Fraunces", fontVariationSettings: '"SOFT" 100, "WONK" 1' },
  { fontFamily: "Inter", fontVariationSettings: '"wght" 250' },
  { fontFamily: "Inter", fontVariationSettings: '"wght" 850' },
  { fontFamily: "Inter", fontStyle: "italic" },
  { fontFamily: "Fraunces", fontStyle: "italic", fontWeight: 300 },
];

export function Case() {
  return (
    <div style={{ position: "absolute", left: 24, top: 8, fontSize: 34, lineHeight: 1.3 }}>
      {VARIATIONS.map((style, i) => (
        <div key={i} style={style}>
          Variable faces
        </div>
      ))}
    </div>
  );
}
