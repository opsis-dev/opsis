const COLORS = [
  "#f8fafc",
  "#f97316",
  "rgba(248,250,252,0.5)",
  "oklch(0.75 0.18 150)",
  "color-mix(in oklch, #f97316 40%, #0ea5e9)",
  "currentColor",
  "hsl(280 80% 70%)",
  "transparent",
];

export function Case() {
  return (
    <div
      style={{
        position: "absolute",
        left: 24,
        top: 16,
        color: "#facc15",
        fontSize: 28,
        fontWeight: 700,
        lineHeight: 1.15,
      }}
    >
      {COLORS.map((color) => (
        <div
          key={color}
          style={{ color, outline: color === "transparent" ? "1px solid #475569" : "none" }}
        >
          Coloured words
        </div>
      ))}
    </div>
  );
}
