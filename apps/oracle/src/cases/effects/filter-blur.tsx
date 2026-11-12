import { SAMPLE, SLATE, cell } from "#oracle/parts.tsx";

const BLURS = [0, 0.5, 1, 2, 4, 8, 16, 40];

export function Case() {
  return (
    <>
      {BLURS.map((radius, i) => (
        <div key={i} style={cell(i, { background: SLATE })}>
          <div style={{ position: "absolute", inset: 16, filter: `blur(${radius}px)` }}>
            <div style={{ width: 60, height: 40, background: "#f97316" }} />
            <div style={{ fontWeight: 700, fontSize: 18, marginTop: 8 }}>{SAMPLE}</div>
          </div>
        </div>
      ))}
    </>
  );
}
