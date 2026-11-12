import { cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <svg width="0" height="0" style={{ position: "absolute" }}>
        <defs>
          <clipPath id="oracle-star" clipPathUnits="objectBoundingBox">
            <path d="M0.5 0 L0.62 0.38 L1 0.38 L0.69 0.6 L0.81 1 L0.5 0.76 L0.19 1 L0.31 0.6 L0 0.38 L0.38 0.38 Z" />
          </clipPath>
          <clipPath id="oracle-blobs">
            <circle cx="40" cy="40" r="35" />
            <circle cx="100" cy="100" r="45" />
          </clipPath>
          <clipPath id="oracle-text">
            <text x="4" y="90" fontSize="72" fontWeight="900" fontFamily="Inter">
              Op
            </text>
          </clipPath>
        </defs>
      </svg>
      <div
        style={cell(0, {
          clipPath: "url(#oracle-star)",
          background: "linear-gradient(#facc15, #f97316)",
        })}
      />
      <div
        style={cell(1, {
          clipPath: "url(#oracle-blobs)",
          background: "linear-gradient(90deg, #0ea5e9, #7c3aed)",
        })}
      />
      <div
        style={cell(2, {
          clipPath: "url(#oracle-text)",
          background: "linear-gradient(#22c55e, #0ea5e9)",
        })}
      />
      <div
        style={cell(3, {
          clipPath: "url(#oracle-star)",
          background: "#f8fafc",
          transform: "rotate(20deg) scale(0.8)",
        })}
      />
    </>
  );
}
