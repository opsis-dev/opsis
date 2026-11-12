import { COOL, LEAF, ROSE, SLATE, WARM, cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE, fontSize: 10 })}>
        <div style={{ width: "5em", height: "3em", background: WARM }} />
      </div>
      <div style={cell(1, { background: SLATE, fontSize: 20 })}>
        <div style={{ width: "5em", height: "3em", background: COOL }} />
      </div>
      <div style={cell(2, { background: SLATE, fontSize: 20 })}>
        <div style={{ fontSize: "0.5em" }}>
          <div style={{ width: "5em", height: "3em", background: LEAF }} />
        </div>
      </div>
      <div style={cell(3, { background: SLATE })}>
        <div style={{ width: "5rem", height: "2rem", background: ROSE }} />
      </div>
      <div style={cell(4, { background: SLATE })}>
        <div style={{ width: "25vw", height: "10vh", background: WARM }} />
      </div>
      <div style={cell(5, { background: SLATE, fontSize: 24 })}>
        <div
          style={{ letterSpacing: "0.5em", lineHeight: "2em", padding: "0.5em", background: COOL }}
        >
          em
        </div>
      </div>
      <div style={cell(6, { background: SLATE })}>
        <div style={{ width: "75%", height: "50%", padding: "10%", background: LEAF }}>
          <div style={{ width: "50%", height: "100%", background: ROSE }} />
        </div>
      </div>
      <div style={cell(7, { background: SLATE, fontSize: 18 })}>
        <div style={{ width: "4ch", background: WARM, fontFamily: "JetBrains Mono" }}>abcd</div>
        <div style={{ width: "4ex", height: "2lh", background: COOL }} />
      </div>
    </>
  );
}
