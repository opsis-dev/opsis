import type { CSSProperties } from "react";

const NUMBERS = ["1,111.11", "8,888.88", "4,710.09", "$0.50"];

const VARIANTS: CSSProperties["fontVariantNumeric"][] = [
  "normal",
  "tabular-nums",
  "slashed-zero",
  "tabular-nums slashed-zero",
];

export function Case() {
  return (
    <div
      style={{ position: "absolute", left: 24, top: 24, display: "flex", gap: 24, fontSize: 22 }}
    >
      {VARIANTS.map((fontVariantNumeric) => (
        <div key={fontVariantNumeric} style={{ fontVariantNumeric, textAlign: "right" }}>
          {NUMBERS.map((n) => (
            <div key={n}>{n}</div>
          ))}
          <div style={{ fontFamily: "Geist" }}>10:01</div>
          <div style={{ fontFeatureSettings: '"ss01"' }}>a1l0</div>
        </div>
      ))}
    </div>
  );
}
