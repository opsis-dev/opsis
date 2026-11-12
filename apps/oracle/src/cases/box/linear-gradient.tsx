import { cell } from "#oracle/parts.tsx";

const GRADIENTS = [
  "linear-gradient(#f97316, #0ea5e9)",
  "linear-gradient(90deg, #f97316, #7c3aed 40%, #22c55e)",
  "linear-gradient(135deg, rgba(255,255,255,0.9), rgba(255,255,255,0) 70%)",
  "linear-gradient(to top right, #0f172a, #f43f5e)",
  "linear-gradient(in oklch, #f97316, #0ea5e9)",
  "linear-gradient(in oklab 45deg, #7c3aed, #facc15)",
  "linear-gradient(180deg, #22c55e 0 30%, #0ea5e9 30% 60%, #f43f5e 60%)",
  "linear-gradient(200deg, transparent, #000 80%), linear-gradient(90deg, #f97316, #7c3aed)",
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
