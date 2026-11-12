import { cell } from "#oracle/parts.tsx";

const CLIPS = [
  "inset(10px)",
  "inset(10% 20% 30% 0 round 16px)",
  "inset(0 50% 0 0)",
  "polygon(50% 0, 100% 100%, 0 100%)",
  "polygon(0 0, 100% 0, 100% 70%, 0 100%)",
  "circle(40% at 50% 50%)",
  "circle(60px at 0 0)",
  "path('M10 70 C 40 0, 90 0, 120 70 S 60 140, 10 70 Z')",
];

export function Case() {
  return (
    <>
      {CLIPS.map((clipPath, i) => (
        <div
          key={i}
          style={cell(i, {
            clipPath,
            background: "linear-gradient(135deg, #f97316, #7c3aed 60%, #0ea5e9)",
          })}
        />
      ))}
    </>
  );
}
