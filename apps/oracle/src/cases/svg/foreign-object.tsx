export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <defs>
        <filter id="oracle-fo-lens">
          <feGaussianBlur stdDeviation="1.5" />
        </filter>
      </defs>
      <foreignObject x="24" y="24" width="280" height="136">
        <div
          style={{
            width: "100%",
            height: "100%",
            background: "#334155",
            padding: 12,
            fontSize: 18,
            lineHeight: 1.3,
          }}
        >
          HTML inside SVG, with <b style={{ color: "#f97316" }}>bold</b> text that wraps across the
          box.
        </div>
      </foreignObject>
      <foreignObject x="328" y="24" width="288" height="136" filter="url(#oracle-fo-lens)">
        <div
          style={{
            width: "100%",
            height: "100%",
            background: "#0f766e",
            padding: 12,
            fontFamily: "JetBrains Mono",
            fontSize: 16,
          }}
        >
          $ opsis --version
        </div>
      </foreignObject>
      <g transform="translate(24 200) rotate(-5)">
        <foreignObject width="280" height="120">
          <div
            style={{
              width: "100%",
              height: "100%",
              background: "linear-gradient(90deg, #7c3aed, #0ea5e9)",
              borderRadius: 16,
            }}
          />
        </foreignObject>
      </g>
    </svg>
  );
}
