const PATHS = [
  "M10 120 L60 20 L110 120 Z",
  "M10 70 C 30 10, 90 10, 110 70 S 190 130, 110 130",
  "M10 70 Q 60 0 110 70 T 210 70",
  "M20 20 H 100 V 100 H 20 Z M40 40 V 80 H 80 V 40 Z",
  "M60 10 A 50 50 0 1 1 59.9 10 Z",
  "M10 100 A 40 20 30 0 1 110 60",
  "m10 10 l40 0 l0 40 l-40 0 z m60 0 h40 v40 h-40 z",
  "M10,60 c20,-40 40,40 60,0 s40,40 60,0",
];

export function Case() {
  return (
    <svg width="640" height="360" viewBox="0 0 640 360" style={{ position: "absolute", inset: 0 }}>
      {PATHS.map((d, i) => (
        <g key={i} transform={`translate(${24 + (i % 4) * 152} ${24 + Math.floor(i / 4) * 164})`}>
          <path d={d} fill={i % 2 === 0 ? "#f97316" : "none"} stroke="#f8fafc" strokeWidth={3} />
        </g>
      ))}
    </svg>
  );
}
