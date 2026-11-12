import { ACCENT, COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

const CHIP = { width: 30, height: 30 };

function Chips() {
  return (
    <>
      <div style={{ ...CHIP, background: WARM }} />
      <div style={{ ...CHIP, background: COOL }} />
      <div style={{ ...CHIP, background: LEAF }} />
    </>
  );
}

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE, display: "block" })}>
        <Chips />
      </div>
      <div style={cell(1, { background: SLATE, display: "flex" })}>
        <Chips />
      </div>
      <div style={cell(2, { background: SLATE, display: "grid", gridTemplateColumns: "1fr 1fr" })}>
        <Chips />
      </div>
      <div style={cell(3, { background: SLATE })}>
        <span style={{ background: WARM }}>inline</span>
        <span style={{ background: COOL }}> spans </span>
        <span style={{ background: LEAF }}>wrap along the line</span>
      </div>
      <div style={cell(4, { background: SLATE })}>
        <div style={{ ...CHIP, display: "inline-block", background: WARM }} />
        <div style={{ ...CHIP, display: "inline-block", background: COOL }} />
        <div style={{ ...CHIP, display: "inline-block", background: LEAF }} />
      </div>
      <div style={cell(5, { background: SLATE })}>
        <div style={{ ...CHIP, background: WARM }} />
        <div style={{ ...CHIP, background: COOL, display: "none" }} />
        <div style={{ ...CHIP, background: LEAF }} />
      </div>
      <div style={cell(6, { background: SLATE })}>
        <div style={{ display: "inline-flex", gap: 4, background: ACCENT, padding: 4 }}>
          <Chips />
        </div>
        text after
      </div>
      <div style={cell(7, { background: SLATE })}>
        <div style={{ display: "contents" }}>
          <div style={{ ...CHIP, background: WARM }} />
        </div>
        <div style={{ display: "inline-grid", gridTemplateColumns: "20px 20px", gap: 2 }}>
          <Chips />
        </div>
      </div>
    </>
  );
}
