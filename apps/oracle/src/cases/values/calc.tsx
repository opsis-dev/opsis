import { COOL, LEAF, ROSE, SLATE, WARM, cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE })}>
        <div style={{ width: "calc(100% - 30px)", height: "calc(50% + 10px)", background: WARM }} />
      </div>
      <div style={cell(1, { background: SLATE })}>
        <div
          style={{
            position: "absolute",
            left: "calc(50% - 20px)",
            top: "calc(25% * 2 - 20px)",
            width: 40,
            height: 40,
            background: COOL,
          }}
        />
      </div>
      <div style={cell(2, { background: SLATE, fontSize: 20 })}>
        <div style={{ width: "calc(3em + 10px)", height: "calc(1em * 2)", background: LEAF }} />
      </div>
      <div style={cell(3, { background: SLATE })}>
        <div
          style={{
            margin: "calc(10px + 5%)",
            height: "calc(100% - 2 * (10px + 5%))",
            background: ROSE,
          }}
        />
      </div>
      <div style={cell(4, { background: SLATE })}>
        <div style={{ width: "clamp(20px, 50%, 60px)", height: 40, background: WARM }} />
        <div style={{ width: "min(200px, 80%)", height: 40, background: COOL }} />
        <div style={{ width: "max(10px, 30%)", height: 40, background: LEAF }} />
      </div>
      <div style={cell(5, { background: SLATE })}>
        <div
          style={{
            translate: "calc(100% + 4px) calc(50% - 10px)",
            width: 50,
            height: 50,
            background: ROSE,
          }}
        />
      </div>
    </>
  );
}
