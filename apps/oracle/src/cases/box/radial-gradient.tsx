import { cell } from "#oracle/parts.tsx";

const GRADIENTS = [
  "radial-gradient(#f97316, #0f172a)",
  "radial-gradient(circle, #f8fafc, #7c3aed 60%, transparent 61%)",
  "radial-gradient(circle at 30% 30%, #0ea5e9, transparent 70%)",
  "radial-gradient(ellipse at top, #22c55e, #0f172a 80%)",
  "radial-gradient(closest-side, #f43f5e, #334155)",
  "radial-gradient(farthest-corner at 80% 90%, #facc15, #7c3aed 50%, #0f172a)",
  "radial-gradient(in oklch, #f97316, #0ea5e9)",
  "radial-gradient(120px 60px at 50% 50%, rgba(255,255,255,0.8), rgba(255,255,255,0))",
];

export function Case() {
  return (
    <>
      {GRADIENTS.map((background, i) => (
        <div key={i} style={cell(i, { background })} />
      ))}
    </>
  );
}
