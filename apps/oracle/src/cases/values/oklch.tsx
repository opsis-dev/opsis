const LIGHTNESS = [0.3, 0.45, 0.6, 0.75, 0.9];
const HUES = [0, 45, 90, 150, 210, 260, 320];

export function Case() {
  return (
    <div
      style={{
        position: "absolute",
        inset: 24,
        display: "grid",
        gridTemplateColumns: `repeat(${HUES.length}, 1fr)`,
        gap: 4,
      }}
    >
      {LIGHTNESS.flatMap((l) =>
        HUES.map((h) => (
          <div key={`${l}-${h}`} style={{ background: `oklch(${l} 0.2 ${h})`, minHeight: 40 }} />
        )),
      )}
      <div
        style={{
          gridColumn: "1 / -1",
          background:
            "linear-gradient(90deg, oklch(0.7 0.37 30), oklch(0.7 0.37 150) 50%, oklch(0.7 0.37 270))",
        }}
      />
      <div
        style={{
          gridColumn: "1 / -1",
          background: "linear-gradient(90deg, oklch(0.7 0.1 30 / 0.2), oklch(0.95 0.05 90 / 1))",
        }}
      />
    </div>
  );
}
