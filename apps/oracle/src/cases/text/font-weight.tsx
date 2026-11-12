const WEIGHTS = [100, 200, 300, 400, 500, 600, 700, 800, 900, 450, 650];

export function Case() {
  return (
    <div
      style={{
        position: "absolute",
        left: 24,
        top: 16,
        display: "grid",
        gridTemplateColumns: "1fr 1fr",
        gap: "4px 32px",
      }}
    >
      {WEIGHTS.map((fontWeight) => (
        <div key={fontWeight} style={{ fontWeight, fontSize: 22, lineHeight: 1.1 }}>
          Weight {fontWeight}
        </div>
      ))}
      <div style={{ fontWeight: 700, fontFamily: "JetBrains Mono", fontSize: 20 }}>mono bold</div>
      <div style={{ fontWeight: 800, fontFamily: "Fraunces", fontSize: 22 }}>serif 800</div>
    </div>
  );
}
