const STOPS = [-0.3, -0.15, 0, 0.15, 0.3];

export function Case() {
  return (
    <div
      style={{
        position: "absolute",
        inset: 24,
        display: "grid",
        gridTemplateColumns: "repeat(5, 1fr)",
        gap: 4,
      }}
    >
      {STOPS.flatMap((a) =>
        STOPS.map((b) => (
          <div key={`${a}-${b}`} style={{ background: `oklab(0.7 ${a} ${b})`, minHeight: 40 }} />
        )),
      )}
      <div
        style={{
          gridColumn: "1 / -1",
          minHeight: 30,
          background: "linear-gradient(in oklab 90deg, #0000ff, #ffff00)",
        }}
      />
      <div
        style={{
          gridColumn: "1 / -1",
          minHeight: 30,
          background: "linear-gradient(in srgb 90deg, #0000ff, #ffff00)",
        }}
      />
    </div>
  );
}
