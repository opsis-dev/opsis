export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <defs>
        <clipPath id="oracle-svg-circle">
          <circle cx="92" cy="90" r="55" />
        </clipPath>
        <clipPath id="oracle-svg-bbox" clipPathUnits="objectBoundingBox">
          <rect x="0.1" y="0.1" width="0.5" height="0.8" rx="0.1" />
        </clipPath>
        <clipPath id="oracle-svg-text">
          <text x="340" y="130" fontSize="96" fontWeight="900" fontFamily="Inter">
            Clip
          </text>
        </clipPath>
        <clipPath id="oracle-svg-evenodd">
          <path d="M40 220 h120 v120 h-120 z M70 250 h60 v60 h-60 z" clipRule="evenodd" />
        </clipPath>
      </defs>
      <g clipPath="url(#oracle-svg-circle)">
        <rect x="24" y="24" width="136" height="136" fill="#f97316" />
        <rect x="24" y="80" width="136" height="20" fill="#0f172a" />
      </g>
      <rect
        x="176"
        y="24"
        width="136"
        height="136"
        fill="#0ea5e9"
        clipPath="url(#oracle-svg-bbox)"
      />
      <rect
        x="328"
        y="24"
        width="288"
        height="136"
        fill="#22c55e"
        clipPath="url(#oracle-svg-text)"
      />
      <rect
        x="24"
        y="200"
        width="160"
        height="160"
        fill="#7c3aed"
        clipPath="url(#oracle-svg-evenodd)"
      />
      <g clipPath="url(#oracle-svg-circle)" transform="translate(250 170)">
        <rect x="24" y="24" width="136" height="136" fill="#f43f5e" />
      </g>
    </svg>
  );
}
