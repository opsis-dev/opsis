import { cell } from "#oracle/parts.tsx";

const GRADIENTS = [
  "repeating-linear-gradient(45deg, #f97316 0 10px, #0f172a 10px 20px)",
  "repeating-linear-gradient(90deg, rgba(255,255,255,0.15) 0 1px, transparent 1px 12px)",
  "repeating-radial-gradient(circle, #0ea5e9 0 6px, #0f172a 6px 14px)",
  "repeating-linear-gradient(0deg, #22c55e, #7c3aed 25%)",
  "repeating-linear-gradient(transparent 0 3px, rgba(0,0,0,0.5) 3px 4px), linear-gradient(#f43f5e, #facc15)",
  "repeating-radial-gradient(ellipse at 20% 20%, #f8fafc 0 2px, transparent 2px 9px)",
  "repeating-linear-gradient(-30deg, #334155 0 8px, #475569 8px 16px)",
  "repeating-conic-gradient(from 0deg at 50% 50%, #0ea5e9 0 15deg, #0f172a 15deg 30deg)",
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
