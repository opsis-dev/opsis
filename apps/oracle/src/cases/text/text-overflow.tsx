import { PARAGRAPH, SLATE, cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <div
        style={cell(0, {
          background: SLATE,
          height: 30,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        })}
      >
        {PARAGRAPH}
      </div>
      <div
        style={cell(1, {
          background: SLATE,
          height: 30,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "clip",
        })}
      >
        {PARAGRAPH}
      </div>
      <div
        style={cell(2, {
          background: SLATE,
          height: 30,
          fontSize: 22,
          fontWeight: 700,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        })}
      >
        {PARAGRAPH}
      </div>
      <div
        style={cell(3, {
          background: SLATE,
          height: 30,
          fontFamily: "JetBrains Mono",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        })}
      >
        {PARAGRAPH}
      </div>
      <div
        style={cell(4, {
          background: SLATE,
          display: "-webkit-box",
          WebkitLineClamp: 3,
          WebkitBoxOrient: "vertical",
          overflow: "hidden",
          fontSize: 15,
        })}
      >
        {PARAGRAPH} {PARAGRAPH}
      </div>
      <div style={cell(5, { background: SLATE, display: "flex", height: 30 })}>
        <span
          style={{
            minWidth: 0,
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          {PARAGRAPH}
        </span>
      </div>
    </>
  );
}
