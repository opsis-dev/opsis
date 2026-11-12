export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <defs>
        <linearGradient id="oracle-fade">
          <stop offset="0" stopColor="#fff" />
          <stop offset="1" stopColor="#fff" stopOpacity="0" />
        </linearGradient>
        <mask id="oracle-mask-fade">
          <rect x="0" y="0" width="640" height="360" fill="url(#oracle-fade)" />
        </mask>
        <mask id="oracle-mask-hole">
          <rect x="0" y="0" width="640" height="360" fill="#fff" />
          <circle cx="400" cy="90" r="40" fill="#000" />
          <circle cx="480" cy="90" r="30" fill="#808080" />
        </mask>
        <mask id="oracle-mask-bbox" maskContentUnits="objectBoundingBox">
          <rect
            x="0"
            y="0"
            width="1"
            height="1"
            fill="url(#oracle-fade)"
            transform="rotate(90 0.5 0.5)"
          />
        </mask>
      </defs>
      <rect x="24" y="24" width="280" height="136" fill="#f97316" mask="url(#oracle-mask-fade)" />
      <rect x="328" y="24" width="288" height="136" fill="#0ea5e9" mask="url(#oracle-mask-hole)" />
      <rect x="24" y="200" width="280" height="136" fill="#22c55e" mask="url(#oracle-mask-bbox)" />
      <g mask="url(#oracle-mask-fade)">
        <text x="328" y="290" fontSize="72" fontWeight="900" fontFamily="Inter" fill="#f8fafc">
          Mask
        </text>
      </g>
    </svg>
  );
}
