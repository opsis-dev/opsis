import { SAMPLE, cell } from "#oracle/parts.tsx";

const FAMILIES = [
  "Inter",
  "JetBrains Mono",
  "Fraunces",
  "Instrument Serif",
  "Caveat",
  "Geist",
  "Space Grotesk",
  "Playfair Display",
];

export function Case() {
  return (
    <>
      {FAMILIES.map((fontFamily, i) => (
        <div key={i} style={cell(i, { fontFamily, fontSize: 22, lineHeight: 1.2 })}>
          {SAMPLE} 0123
        </div>
      ))}
    </>
  );
}
