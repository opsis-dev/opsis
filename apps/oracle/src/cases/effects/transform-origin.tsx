import { SLATE, cell } from "#oracle/parts.tsx";

const ORIGINS = [
  "center",
  "0 0",
  "100% 100%",
  "left bottom",
  "20px 80px",
  "50% 0",
  "right center",
  "-20px -20px",
];

export function Case() {
  return (
    <>
      {ORIGINS.map((transformOrigin, i) => (
        <div key={i} style={cell(i, { background: SLATE })}>
          <div
            style={{
              position: "absolute",
              left: 33,
              top: 39,
              width: 70,
              height: 70,
              outline: "1px dashed #94a3b8",
            }}
          />
          <div
            style={{
              position: "absolute",
              left: 33,
              top: 39,
              width: 70,
              height: 70,
              background: "rgba(124,58,237,0.85)",
              transformOrigin,
              transform: "rotate(25deg) scale(0.7)",
            }}
          />
        </div>
      ))}
    </>
  );
}
