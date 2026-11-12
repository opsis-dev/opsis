import { SLATE, cell } from "#oracle/parts.tsx";

const SHADOWS = [
  "drop-shadow(0 4px 6px rgba(0,0,0,0.8))",
  "drop-shadow(6px 6px 0 #f97316)",
  "drop-shadow(0 0 12px #0ea5e9)",
  "drop-shadow(0 0 2px #fff) drop-shadow(0 0 10px #7c3aed)",
  "drop-shadow(-8px 10px 4px rgba(34,197,94,0.7))",
  "drop-shadow(0 20px 20px rgba(0,0,0,0.9)) brightness(1.2)",
  "blur(2px) drop-shadow(0 0 6px #facc15)",
  "drop-shadow(0 0 0 transparent)",
];

export function Case() {
  return (
    <>
      {SHADOWS.map((filter, i) => (
        <div key={i} style={cell(i, { background: SLATE, display: "grid", placeItems: "center" })}>
          <div style={{ filter, display: "grid", placeItems: "center" }}>
            <svg width="70" height="70" viewBox="0 0 70 70">
              <path d="M35 4 L66 62 L4 62 Z" fill="#f8fafc" />
            </svg>
            <span style={{ fontWeight: 800, fontSize: 20 }}>Hi</span>
          </div>
        </div>
      ))}
    </>
  );
}
