import { SLATE, cell } from "#oracle/parts.tsx";

const LONG =
  "A demo film takes a recording, a camera that follows the cursor, captions, music and titles, and puts them in order.";

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE, fontSize: 13, height: "auto", width: 136 })}>
        {LONG}
      </div>
      <div style={cell(1, { background: SLATE, fontSize: 16, width: 100 })}>{LONG}</div>
      <div style={cell(2, { background: SLATE, fontSize: 12, fontFamily: "JetBrains Mono" })}>
        {LONG}
      </div>
      <div style={cell(3, { background: SLATE, fontSize: 15, hyphens: "manual" })}>
        Extra&shy;ordinary in&shy;com&shy;pre&shy;hen&shy;si&shy;bil&shy;i&shy;ties
      </div>
      <div style={cell(4, { background: SLATE, fontSize: 14 })}>
        Mixed scripts: Grüße, Ελληνικά, Кириллица, Ærøskøbing, ½ → ≥
      </div>
      <div style={cell(5, { background: SLATE, fontSize: 18, wordBreak: "keep-all" })}>
        Long­words­without­spaces stay
      </div>
      <div style={cell(6, { background: SLATE, fontSize: 14, textWrap: "balance" })}>{LONG}</div>
      <div
        style={cell(7, {
          background: SLATE,
          fontSize: 30,
          fontWeight: 800,
          lineHeight: 1,
          letterSpacing: "-0.04em",
        })}
      >
        Big headline wraps
      </div>
    </>
  );
}
