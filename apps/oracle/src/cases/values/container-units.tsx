import { COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE, containerType: "size" })}>
        <div style={{ width: "50cqw", height: "25cqh", background: WARM }} />
      </div>
      <div style={cell(1, { background: SLATE, containerType: "size" })}>
        <div style={{ fontSize: "20cqw", fontWeight: 800, lineHeight: 1 }}>Fit</div>
      </div>
      <div style={cell(2, { background: SLATE, containerType: "size" })}>
        <div style={{ width: "40cqmin", height: "40cqmax", background: COOL }} />
      </div>
      <div style={cell(3, { background: SLATE, containerType: "inline-size", height: "auto" })}>
        <div style={{ padding: "5cqi", fontSize: "12cqi", background: LEAF }}>inline</div>
      </div>
      <div style={cell(4, { background: SLATE, containerType: "size", width: 280, height: 148 })}>
        <div style={{ fontSize: "15cqh", fontWeight: 700, marginLeft: "10cqw" }}>Wide box</div>
      </div>
    </>
  );
}
