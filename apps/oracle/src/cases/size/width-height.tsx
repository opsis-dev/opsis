import { ACCENT, COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE })}>
        <div style={{ width: 80, height: 40, background: WARM }} />
      </div>
      <div style={cell(1, { background: SLATE })}>
        <div style={{ width: "50%", height: "75%", background: COOL }} />
      </div>
      <div style={cell(2, { background: SLATE })}>
        <div style={{ background: LEAF, padding: 8 }}>auto</div>
      </div>
      <div style={cell(3, { background: SLATE, display: "flex" })}>
        <div style={{ height: "100%", width: "calc(100% - 40px)", background: ACCENT }} />
      </div>
      <div style={cell(4, { background: SLATE })}>
        <div style={{ width: "100%", height: 12.5, background: WARM }} />
        <div style={{ width: 99.5, height: 20.25, background: COOL }} />
      </div>
      <div style={cell(5, { background: SLATE })}>
        <span style={{ width: 100, height: 100, background: LEAF }}>inline</span>
      </div>
      <div style={cell(6, { background: SLATE, padding: 16 })}>
        <div style={{ height: "100%", background: ACCENT }} />
      </div>
      <div style={cell(7, { background: SLATE, aspectRatio: "auto" })}>
        <div style={{ width: 120, aspectRatio: "16 / 9", background: COOL }} />
      </div>
    </>
  );
}
