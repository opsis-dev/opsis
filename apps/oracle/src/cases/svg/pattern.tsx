export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <defs>
        <pattern id="oracle-dots" width="16" height="16" patternUnits="userSpaceOnUse">
          <circle cx="8" cy="8" r="3" fill="#f8fafc" />
        </pattern>
        <pattern
          id="oracle-stripes"
          width="12"
          height="12"
          patternUnits="userSpaceOnUse"
          patternTransform="rotate(45)"
        >
          <rect width="6" height="12" fill="#f97316" />
        </pattern>
        <pattern
          id="oracle-grid"
          width="0.25"
          height="0.25"
          patternContentUnits="objectBoundingBox"
        >
          <rect width="0.24" height="0.24" fill="#0ea5e9" />
        </pattern>
      </defs>
      <rect x="24" y="24" width="280" height="136" fill="url(#oracle-dots)" />
      <rect x="328" y="24" width="288" height="136" fill="url(#oracle-stripes)" />
      <rect x="24" y="200" width="280" height="136" fill="url(#oracle-grid)" />
      <circle
        cx="470"
        cy="268"
        r="68"
        fill="url(#oracle-stripes)"
        stroke="url(#oracle-dots)"
        strokeWidth="16"
      />
    </svg>
  );
}
