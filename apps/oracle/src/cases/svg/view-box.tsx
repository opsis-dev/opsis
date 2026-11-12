import { SLATE, cell } from "#oracle/parts.tsx";

const RATIOS = [
  "xMidYMid meet",
  "xMinYMin meet",
  "xMaxYMax meet",
  "xMidYMid slice",
  "none",
  "xMinYMid slice",
];

function Art() {
  return (
    <>
      <rect x="0" y="0" width="100" height="50" fill="#f97316" />
      <circle cx="50" cy="25" r="20" fill="#0ea5e9" />
      <path d="M0 50 L50 0 L100 50" fill="none" stroke="#f8fafc" strokeWidth="3" />
    </>
  );
}

export function Case() {
  return (
    <>
      {RATIOS.map((preserveAspectRatio, i) => (
        <div key={i} style={cell(i, { background: SLATE })}>
          <svg
            width="136"
            height="148"
            viewBox="0 0 100 50"
            preserveAspectRatio={preserveAspectRatio}
          >
            <Art />
          </svg>
        </div>
      ))}
      <div style={cell(6, { background: SLATE })}>
        <svg viewBox="-50 -25 200 100" style={{ width: "100%", height: "100%" }}>
          <Art />
        </svg>
      </div>
      <div style={cell(7, { background: SLATE })}>
        <svg width="136" height="148" viewBox="0 0 10 5" overflow="visible">
          <g transform="scale(0.1)">
            <Art />
          </g>
        </svg>
      </div>
    </>
  );
}
