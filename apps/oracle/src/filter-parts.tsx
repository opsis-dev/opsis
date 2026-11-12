import type { ReactNode } from "react";

export function Subject() {
  return (
    <>
      <rect x="10" y="10" width="80" height="60" fill="#f97316" />
      <circle cx="90" cy="80" r="35" fill="#0ea5e9" />
      <text x="12" y="125" fontSize="26" fontWeight="800" fontFamily="Inter" fill="#f8fafc">
        Filter
      </text>
    </>
  );
}

export function Filtered({ filters }: { filters: [string, ReactNode][] }) {
  return (
    <svg width="640" height="360" style={{ position: "absolute", inset: 0 }}>
      <defs>
        {filters.map(([id, primitives]) => (
          <filter key={id} id={id} x="-20%" y="-20%" width="140%" height="140%">
            {primitives}
          </filter>
        ))}
      </defs>
      {filters.map(([id], i) => (
        <g key={id} transform={`translate(${24 + (i % 4) * 152} ${24 + Math.floor(i / 4) * 164})`}>
          <rect width="136" height="148" fill="#334155" />
          <g filter={`url(#${id})`}>
            <Subject />
          </g>
        </g>
      ))}
    </svg>
  );
}
