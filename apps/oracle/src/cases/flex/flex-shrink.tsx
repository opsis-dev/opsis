import { COOL, LEAF, PARAGRAPH, SLATE, WARM, cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE, display: "flex" })}>
        <div style={{ width: 100, height: 30, background: WARM }} />
        <div style={{ width: 100, height: 30, background: COOL }} />
      </div>
      <div style={cell(1, { background: SLATE, display: "flex" })}>
        <div style={{ width: 100, height: 30, background: WARM, flexShrink: 0 }} />
        <div style={{ width: 100, height: 30, background: COOL }} />
      </div>
      <div style={cell(2, { background: SLATE, display: "flex" })}>
        <div style={{ width: 100, height: 30, background: WARM, flexShrink: 3 }} />
        <div style={{ width: 100, height: 30, background: COOL, flexShrink: 1 }} />
      </div>
      <div style={cell(3, { background: SLATE, display: "flex", fontSize: 12 })}>
        <div style={{ background: WARM }}>{PARAGRAPH}</div>
        <div style={{ background: COOL, flexShrink: 0 }}>fixed</div>
      </div>
      <div style={cell(4, { background: SLATE, display: "flex", fontSize: 12 })}>
        <div style={{ background: LEAF, minWidth: 0, whiteSpace: "nowrap", overflow: "hidden" }}>
          {PARAGRAPH}
        </div>
        <div style={{ background: COOL, flexShrink: 0 }}>end</div>
      </div>
      <div style={cell(5, { background: SLATE, display: "flex", flexDirection: "column" })}>
        <div style={{ height: 100, background: WARM }} />
        <div style={{ height: 100, background: COOL, flexShrink: 0 }} />
      </div>
      <div style={cell(6, { background: SLATE, display: "flex" })}>
        <div style={{ flexBasis: 120, height: 30, background: WARM }} />
        <div style={{ flexBasis: 60, height: 30, background: LEAF }} />
      </div>
      <div style={cell(7, { background: SLATE, display: "flex", fontSize: 12 })}>
        <div style={{ background: WARM, wordBreak: "break-all" }}>Supercalifragilistic</div>
        <div style={{ background: COOL }}>Unbreakableword</div>
      </div>
    </>
  );
}
