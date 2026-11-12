const CAPS = ["butt", "round", "square"] as const;
const JOINS = ["miter", "round", "bevel"] as const;

export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      {CAPS.map((cap, i) => (
        <line
          key={cap}
          x1={40}
          y1={40 + i * 40}
          x2={240}
          y2={40 + i * 40}
          stroke="#f97316"
          strokeWidth={20}
          strokeLinecap={cap}
        />
      ))}
      {JOINS.map((join, i) => (
        <polyline
          key={join}
          points={`${300 + i * 110},140 ${340 + i * 110},40 ${380 + i * 110},140`}
          fill="none"
          stroke="#0ea5e9"
          strokeWidth={18}
          strokeLinejoin={join}
        />
      ))}
      <path d="M40 220 C 120 160, 200 300, 280 220" fill="none" stroke="#22c55e" strokeWidth={1} />
      <path
        d="M40 260 C 120 200, 200 340, 280 260"
        fill="none"
        stroke="#22c55e"
        strokeWidth={2.5}
      />
      <path
        d="M40 300 C 120 240, 200 380, 280 300"
        fill="none"
        stroke="#22c55e"
        strokeWidth={8}
        strokeLinecap="round"
      />
      <rect
        x="320"
        y="200"
        width="120"
        height="120"
        fill="#7c3aed"
        stroke="#f8fafc"
        strokeWidth={10}
        strokeOpacity={0.5}
        fillOpacity={0.6}
      />
      <circle
        cx="540"
        cy="260"
        r="50"
        fill="none"
        stroke="#f43f5e"
        strokeWidth={12}
        strokeMiterlimit={1}
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
}
