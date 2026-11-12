export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }} fontFamily="Inter">
      <text x="24" y="60" fontSize="36" fill="#f8fafc">
        Start anchored
      </text>
      <text x="320" y="120" fontSize="36" fill="#f97316" textAnchor="middle">
        Middle anchored
      </text>
      <text x="616" y="180" fontSize="36" fill="#0ea5e9" textAnchor="end" fontWeight="700">
        End anchored
      </text>
      <text x="24" y="240" fontSize="28" fill="#22c55e" letterSpacing="4">
        Spaced{" "}
        <tspan fill="#f43f5e" fontWeight="800">
          tspan
        </tspan>{" "}
        run
      </text>
      <text
        x="400"
        y="250"
        fontSize="24"
        fill="#facc15"
        dominantBaseline="middle"
        transform="rotate(-10 400 250)"
      >
        Rotated
      </text>
      <text
        x="24"
        y="320"
        fontSize="40"
        fill="none"
        stroke="#f8fafc"
        strokeWidth="1.5"
        fontFamily="JetBrains Mono"
      >
        stroke()
      </text>
      <text
        x="400"
        y="320"
        fontSize="32"
        fill="#7c3aed"
        fontStyle="italic"
        fontFamily="Fraunces"
        opacity="0.8"
      >
        serif italic
      </text>
    </svg>
  );
}
