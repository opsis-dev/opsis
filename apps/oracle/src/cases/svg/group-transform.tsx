function Mark() {
  return (
    <>
      <rect x="-30" y="-30" width="60" height="60" fill="#f97316" />
      <circle cx="0" cy="0" r="12" fill="#0f172a" />
    </>
  );
}

export function Case() {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <g transform="translate(80 90)">
        <Mark />
      </g>
      <g transform="translate(230 90) rotate(30)">
        <Mark />
      </g>
      <g transform="translate(380 90) scale(1.5 0.6)">
        <Mark />
      </g>
      <g transform="translate(530 90) skewX(25)">
        <Mark />
      </g>
      <g transform="translate(80 260)" opacity="0.5">
        <Mark />
        <g transform="translate(20 20)">
          <Mark />
        </g>
      </g>
      <g transform="matrix(0.8 0.4 -0.4 0.8 230 260)">
        <Mark />
      </g>
      <g
        transform="translate(380 260)"
        style={{ transform: "translate(380px, 260px) rotate(-20deg)" }}
      >
        <Mark />
      </g>
      <g transform="translate(530 260) rotate(45) translate(20 0)">
        <Mark />
      </g>
    </svg>
  );
}
