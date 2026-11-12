import { COOL, LEAF, PARAGRAPH, SLATE, WARM, cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE })}>
        <div style={{ maxWidth: 90, background: WARM, fontSize: 12 }}>{PARAGRAPH}</div>
      </div>
      <div style={cell(1, { background: SLATE })}>
        <div style={{ width: 20, minWidth: 100, height: 40, background: COOL }} />
      </div>
      <div style={cell(2, { background: SLATE })}>
        <div style={{ width: "100%", maxWidth: "60%", height: 40, background: LEAF }} />
      </div>
      <div style={cell(3, { background: SLATE, display: "flex" })}>
        <div style={{ flex: 1, minWidth: 0, maxWidth: 70, height: 40, background: WARM }} />
        <div style={{ flex: 1, minWidth: 50, height: 40, background: COOL }} />
      </div>
      <div style={cell(4, { background: SLATE })}>
        <div style={{ height: 300, maxHeight: 80, background: LEAF }} />
      </div>
      <div style={cell(5, { background: SLATE })}>
        <div style={{ minHeight: 120, background: WARM }}>short</div>
      </div>
      <div style={cell(6, { background: SLATE, display: "flex", alignItems: "flex-start" })}>
        <div style={{ maxWidth: "min(80px, 50%)", background: COOL, fontSize: 12 }}>
          {PARAGRAPH}
        </div>
      </div>
      <div style={cell(7, { background: SLATE })}>
        <div style={{ width: "max(40px, 75%)", height: 40, background: LEAF }} />
      </div>
    </>
  );
}
