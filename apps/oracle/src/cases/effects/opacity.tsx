import { COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

const LEVELS = [1, 0.75, 0.5, 0.25, 0.1, 0.004, 0];

export function Case() {
  return (
    <>
      {LEVELS.map((opacity, i) => (
        <div key={i} style={cell(i, { background: SLATE })}>
          <div
            style={{
              position: "absolute",
              left: 10,
              top: 10,
              width: 80,
              height: 80,
              background: WARM,
              opacity,
            }}
          />
          <div
            style={{
              position: "absolute",
              left: 46,
              top: 46,
              width: 80,
              height: 80,
              background: COOL,
              opacity,
            }}
          />
        </div>
      ))}
      <div style={cell(7, { background: SLATE })}>
        <div style={{ position: "absolute", inset: 10, opacity: 0.5 }}>
          <div
            style={{
              position: "absolute",
              left: 0,
              top: 0,
              width: 80,
              height: 80,
              background: WARM,
            }}
          />
          <div
            style={{
              position: "absolute",
              left: 36,
              top: 36,
              width: 80,
              height: 80,
              background: LEAF,
            }}
          />
        </div>
      </div>
    </>
  );
}
