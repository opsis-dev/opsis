export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <defs>
        <linearGradient id="oracle-lin">
          <stop offset="0" stopColor="#f97316" />
          <stop offset="1" stopColor="#7c3aed" />
        </linearGradient>
        <linearGradient id="oracle-lin-v" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#0ea5e9" />
          <stop offset="0.5" stopColor="#22c55e" stopOpacity="0.5" />
          <stop offset="1" stopColor="#0ea5e9" stopOpacity="0" />
        </linearGradient>
        <linearGradient
          id="oracle-lin-user"
          gradientUnits="userSpaceOnUse"
          x1="320"
          y1="40"
          x2="420"
          y2="140"
          spreadMethod="reflect"
        >
          <stop offset="0" stopColor="#facc15" />
          <stop offset="0.3" stopColor="#f43f5e" />
        </linearGradient>
        <radialGradient id="oracle-rad">
          <stop offset="0" stopColor="#f8fafc" />
          <stop offset="1" stopColor="#7c3aed" />
        </radialGradient>
        <radialGradient id="oracle-rad-focus" cx="0.5" cy="0.5" r="0.5" fx="0.25" fy="0.25">
          <stop offset="0" stopColor="#facc15" />
          <stop offset="0.7" stopColor="#f97316" />
          <stop offset="1" stopColor="#0f172a" />
        </radialGradient>
        <radialGradient id="oracle-rad-pad" r="0.3" spreadMethod="repeat">
          <stop offset="0" stopColor="#0ea5e9" />
          <stop offset="1" stopColor="#0f172a" />
        </radialGradient>
      </defs>
      <rect x="24" y="40" width="136" height="100" fill="url(#oracle-lin)" />
      <rect x="176" y="40" width="136" height="100" fill="url(#oracle-lin-v)" />
      <rect x="328" y="40" width="136" height="100" fill="url(#oracle-lin-user)" />
      <circle cx="548" cy="90" r="60" fill="none" stroke="url(#oracle-lin)" strokeWidth="14" />
      <circle cx="92" cy="260" r="65" fill="url(#oracle-rad)" />
      <rect x="176" y="200" width="136" height="120" fill="url(#oracle-rad-focus)" />
      <rect x="328" y="200" width="136" height="120" fill="url(#oracle-rad-pad)" />
      <text
        x="480"
        y="280"
        fontSize="64"
        fontWeight="900"
        fontFamily="Inter"
        fill="url(#oracle-lin)"
      >
        Aa
      </text>
    </svg>
  );
}
