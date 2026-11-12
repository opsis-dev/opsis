import { PARAGRAPH, SLATE, cell } from "#oracle/parts.tsx";

const HEIGHTS = [0.9, 1, 1.15, 1.5, 2, "20px", "normal", "150%"];

export function Case() {
  return (
    <>
      {HEIGHTS.map((lineHeight, i) => (
        <div key={i} style={cell(i, { background: SLATE, lineHeight, fontSize: 15 })}>
          {PARAGRAPH}
        </div>
      ))}
    </>
  );
}
