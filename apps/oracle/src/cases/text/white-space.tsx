import type { CSSProperties } from "react";
import { SLATE, cell } from "#oracle/parts.tsx";

const TEXT = "Two  spaces,\na newline and\ta tab in one long line of words";

const MODES: CSSProperties["whiteSpace"][] = [
  "normal",
  "nowrap",
  "pre",
  "pre-wrap",
  "pre-line",
  "break-spaces",
];

export function Case() {
  return (
    <>
      {MODES.map((whiteSpace, i) => (
        <div
          key={i}
          style={cell(i, { background: SLATE, whiteSpace, fontSize: 14, lineHeight: 1.3 })}
        >
          {TEXT}
        </div>
      ))}
      <div style={cell(6, { background: SLATE, fontSize: 14, overflowWrap: "anywhere" })}>
        https://opsis.example/a-very-long-address-without-breaks
      </div>
      <div
        style={cell(7, {
          background: SLATE,
          fontSize: 14,
          fontFamily: "JetBrains Mono",
          whiteSpace: "pre",
        })}
      >
        {"fn main() {\n    run();\n}"}
      </div>
    </>
  );
}
