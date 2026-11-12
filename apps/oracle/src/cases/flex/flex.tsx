import type { CSSProperties } from "react";
import { COOL, LEAF, SLATE, WARM, cell } from "#oracle/parts.tsx";

const SETS: [CSSProperties, CSSProperties, CSSProperties][] = [
  [{ flex: 1 }, { flex: 1 }, { flex: 1 }],
  [{ flex: 1 }, { flex: 2 }, { flex: 3 }],
  [{ flex: "none", width: 40 }, { flex: 1 }, { flex: "none", width: 20 }],
  [{ flex: "1 1 0" }, { flex: "1 1 auto", width: 50 }, { flex: "0 0 30px" }],
  [{ flexGrow: 1 }, {}, { flexGrow: 2 }],
  [{ flex: "auto" }, { flex: "initial", width: 60 }, { flex: "auto" }],
  [{ flex: 1, minWidth: 80 }, { flex: 1 }, { flex: 1 }],
  [{ flex: 0.5 }, { flex: 0.25 }, { flex: 0 }],
];

export function Case() {
  return (
    <>
      {SETS.map(([a, b, c], i) => (
        <div
          key={i}
          style={cell(i, { background: SLATE, display: "flex", alignItems: "flex-start" })}
        >
          <div style={{ height: 40, background: WARM, ...a }} />
          <div style={{ height: 60, background: COOL, ...b }} />
          <div style={{ height: 80, background: LEAF, ...c }} />
        </div>
      ))}
    </>
  );
}
