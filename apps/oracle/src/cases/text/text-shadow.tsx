const SHADOWS = [
  "0 2px 4px rgba(0,0,0,0.9)",
  "0 0 12px #0ea5e9",
  "3px 3px 0 #f97316",
  "0 0 2px #fff, 0 0 10px #f43f5e, 0 0 30px #f43f5e",
  "-2px 0 #0ea5e9, 2px 0 #f43f5e",
];

export function Case() {
  return (
    <div
      style={{
        position: "absolute",
        left: 24,
        top: 16,
        fontSize: 40,
        fontWeight: 800,
        lineHeight: 1.6,
      }}
    >
      {SHADOWS.map((textShadow) => (
        <div key={textShadow} style={{ textShadow }}>
          Shadowed
        </div>
      ))}
    </div>
  );
}
