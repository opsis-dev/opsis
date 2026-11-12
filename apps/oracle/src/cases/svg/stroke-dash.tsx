const DRAWN = [0, 0.1, 0.33, 0.5, 0.75, 1];
const LENGTH = 400;

export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      {DRAWN.map((share, i) => (
        <path
          key={share}
          d={`M${30 + i * 100} 160 C ${60 + i * 100} 20, ${90 + i * 100} 20, ${110 + i * 100} 100 S ${60 + i * 100} 200, ${40 + i * 100} 150`}
          pathLength={LENGTH}
          fill="none"
          stroke="#f97316"
          strokeWidth={6}
          strokeLinecap="round"
          strokeDasharray={LENGTH}
          strokeDashoffset={LENGTH * (1 - share)}
        />
      ))}
      <line
        x1="30"
        y1="230"
        x2="610"
        y2="230"
        stroke="#0ea5e9"
        strokeWidth={4}
        strokeDasharray="12 6"
      />
      <line
        x1="30"
        y1="260"
        x2="610"
        y2="260"
        stroke="#22c55e"
        strokeWidth={4}
        strokeDasharray="2 8 16 8"
        strokeDashoffset={5}
      />
      <circle
        cx="120"
        cy="310"
        r="30"
        fill="none"
        stroke="#f8fafc"
        strokeWidth={4}
        strokeDasharray="4 4"
      />
      <circle
        cx="240"
        cy="310"
        r="30"
        fill="none"
        stroke="#f43f5e"
        strokeWidth={6}
        strokeDasharray="47 200"
        transform="rotate(-90 240 310)"
        strokeLinecap="round"
      />
      <rect
        x="320"
        y="285"
        width="120"
        height="50"
        rx="10"
        fill="none"
        stroke="#facc15"
        strokeWidth={3}
        strokeDasharray="20 10"
      />
    </svg>
  );
}
