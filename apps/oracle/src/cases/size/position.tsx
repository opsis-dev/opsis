import { ACCENT, COOL, LEAF, ROSE, SLATE, WARM, cell } from "#oracle/parts.tsx";

const BOX = { width: 40, height: 40 };

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE })}>
        <div style={{ ...BOX, background: WARM }} />
        <div style={{ ...BOX, background: COOL, position: "relative", left: 30, top: -10 }} />
        <div style={{ ...BOX, background: LEAF }} />
      </div>
      <div style={cell(1, { background: SLATE })}>
        <div style={{ ...BOX, background: WARM, position: "absolute", left: 10, top: 10 }} />
        <div style={{ ...BOX, background: COOL, position: "absolute", right: 10, bottom: 10 }} />
      </div>
      <div style={cell(2, { background: SLATE })}>
        <div
          style={{
            background: LEAF,
            position: "absolute",
            left: 20,
            right: 20,
            top: 30,
            bottom: 50,
          }}
        />
      </div>
      <div style={cell(3, { background: SLATE })}>
        <div
          style={{
            position: "relative",
            left: 20,
            top: 20,
            width: 90,
            height: 90,
            background: ACCENT,
          }}
        >
          <div
            style={{ ...BOX, background: ROSE, position: "absolute", right: -10, bottom: -10 }}
          />
        </div>
      </div>
      <div style={cell(4, { background: SLATE })}>
        <div style={{ ...BOX, background: WARM, position: "absolute", left: "50%", top: "50%" }} />
      </div>
      <div style={cell(5, { background: SLATE })}>
        <div
          style={{
            ...BOX,
            background: COOL,
            position: "absolute",
            left: "50%",
            top: "50%",
            translate: "-50% -50%",
          }}
        />
      </div>
      <div style={cell(6, { background: SLATE, overflow: "hidden" })}>
        <div style={{ ...BOX, background: LEAF, position: "absolute", left: -20, top: -20 }} />
        <div style={{ ...BOX, background: ROSE, position: "absolute", right: -20, bottom: -20 }} />
      </div>
      <div style={cell(7, { background: SLATE })}>
        <div style={{ ...BOX, background: WARM, position: "absolute", right: "10%", top: "10%" }} />
        <div
          style={{
            background: ACCENT,
            position: "absolute",
            left: 10,
            top: 70,
            width: "auto",
            right: 60,
            height: 30,
          }}
        />
      </div>
    </>
  );
}
