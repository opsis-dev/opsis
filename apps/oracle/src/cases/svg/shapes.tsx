export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <circle cx="80" cy="90" r="50" fill="#f97316" />
      <circle cx="230" cy="90" r="50" fill="none" stroke="#0ea5e9" strokeWidth="8" />
      <rect x="320" y="40" width="100" height="100" fill="#22c55e" />
      <rect
        x="460"
        y="40"
        width="140"
        height="100"
        rx="24"
        ry="16"
        fill="#7c3aed"
        stroke="#f8fafc"
        strokeWidth="2"
      />
      <ellipse cx="80" cy="260" rx="60" ry="30" fill="#f43f5e" />
      <line x1="170" y1="210" x2="290" y2="310" stroke="#facc15" strokeWidth="6" />
      <line x1="170" y1="310" x2="290" y2="210" stroke="#facc15" strokeWidth="1" />
      <polygon points="380,210 440,310 320,310" fill="#0ea5e9" />
      <polyline
        points="470,300 500,220 530,280 560,210 600,300"
        fill="none"
        stroke="#f8fafc"
        strokeWidth="4"
      />
      <circle cx="80" cy="260" r="0" fill="#fff" />
      <rect x="320" y="200" width="0" height="10" fill="#fff" />
    </svg>
  );
}
