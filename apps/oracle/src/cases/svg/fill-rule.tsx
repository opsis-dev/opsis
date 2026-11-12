const STAR = "M70 10 L95 130 L10 55 L130 55 L45 130 Z";
const RINGS = "M70 10 a60 60 0 1 0 0.1 0 Z M70 40 a30 30 0 1 0 0.1 0 Z";

export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <path d={STAR} transform="translate(20 20)" fill="#f97316" fillRule="nonzero" />
      <path d={STAR} transform="translate(170 20)" fill="#f97316" fillRule="evenodd" />
      <path d={RINGS} transform="translate(320 20)" fill="#0ea5e9" fillRule="nonzero" />
      <path d={RINGS} transform="translate(470 20)" fill="#0ea5e9" fillRule="evenodd" />
      <path
        d={STAR}
        transform="translate(20 200)"
        fill="#22c55e"
        fillOpacity="0.5"
        stroke="#f8fafc"
        strokeOpacity="0.5"
        strokeWidth="6"
      />
      <path
        d={STAR}
        transform="translate(170 200)"
        fill="none"
        stroke="#f43f5e"
        strokeWidth="4"
        strokeLinejoin="round"
      />
      <g opacity="0.5" transform="translate(320 200)">
        <path d={STAR} fill="#facc15" />
        <path d={STAR} transform="translate(20 0)" fill="#7c3aed" />
      </g>
      <path
        d={RINGS}
        transform="translate(470 200)"
        fill="#22c55e"
        fillRule="evenodd"
        style={{ mixBlendMode: "screen" }}
      />
    </svg>
  );
}
