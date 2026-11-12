import { cell } from "#oracle/parts.tsx";

const GRADIENTS = [
  "conic-gradient(#f97316, #0ea5e9, #22c55e, #f97316)",
  "conic-gradient(from 90deg, #7c3aed, #f43f5e)",
  "conic-gradient(from 0deg at 30% 70%, #facc15, #0f172a)",
  "conic-gradient(#22c55e 0 25%, #334155 25% 100%)",
  "conic-gradient(in oklch, #f97316, #0ea5e9, #f97316)",
  "conic-gradient(from 45deg, rgba(255,255,255,0.9), rgba(255,255,255,0) 50%, rgba(255,255,255,0.9))",
  "repeating-conic-gradient(#f8fafc 0 10deg, #0f172a 10deg 20deg)",
  "conic-gradient(from 180deg at 50% 0%, #0ea5e9, #7c3aed, #0ea5e9)",
];

export function Case() {
  return (
    <>
      {GRADIENTS.map((background, i) => (
        <div key={i} style={cell(i, { background, borderRadius: i % 2 === 0 ? "50%" : 0 })} />
      ))}
    </>
  );
}
