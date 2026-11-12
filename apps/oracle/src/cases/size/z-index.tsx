import type { CSSProperties } from "react";
import { ACCENT, COOL, LEAF, ROSE, SLATE, WARM, cell } from "#oracle/parts.tsx";

function square(
  left: number,
  top: number,
  background: string,
  extra: CSSProperties = {},
): CSSProperties {
  return { position: "absolute", left, top, width: 60, height: 60, background, ...extra };
}

export function Case() {
  return (
    <>
      <div style={cell(0, { background: SLATE })}>
        <div style={square(10, 10, WARM)} />
        <div style={square(40, 40, COOL)} />
        <div style={square(70, 70, LEAF)} />
      </div>
      <div style={cell(1, { background: SLATE })}>
        <div style={square(10, 10, WARM, { zIndex: 3 })} />
        <div style={square(40, 40, COOL, { zIndex: 2 })} />
        <div style={square(70, 70, LEAF, { zIndex: 1 })} />
      </div>
      <div style={cell(2, { background: SLATE, isolation: "isolate" })}>
        <div style={square(20, 20, WARM)} />
        <div style={square(50, 50, COOL, { zIndex: -1 })} />
      </div>
      <div style={cell(3, { background: SLATE })}>
        <div style={square(10, 10, WARM, { zIndex: 1, opacity: 0.99 })}>
          <div style={square(30, 30, ROSE, { zIndex: 10 })} />
        </div>
        <div style={square(40, 40, COOL, { zIndex: 2 })} />
      </div>
      <div style={cell(4, { background: SLATE })}>
        <div style={square(10, 10, WARM, { transform: "translateZ(0)" })} />
        <div style={square(40, 40, COOL)} />
      </div>
      <div style={cell(5, { background: SLATE })}>
        <div style={square(40, 40, COOL, { position: "relative" })} />
        <div style={{ ...square(10, 10, WARM), position: "static", marginTop: 0 }} />
      </div>
      <div style={cell(6, { background: SLATE })}>
        <div style={square(10, 10, WARM, { zIndex: 5 })} />
        <div style={square(30, 30, ACCENT, { zIndex: 5 })} />
        <div style={square(50, 50, LEAF, { zIndex: 4 })} />
      </div>
      <div style={cell(7, { background: SLATE })}>
        <div style={square(10, 10, WARM, { filter: "blur(0px)" })} />
        <div style={square(40, 40, COOL, { mixBlendMode: "normal", zIndex: 0 })} />
        <div style={square(25, 70, ROSE, { zIndex: "auto" })} />
      </div>
    </>
  );
}
