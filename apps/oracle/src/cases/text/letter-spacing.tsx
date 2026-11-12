const SPACINGS = ["-0.05em", "-0.02em", "0", "0.02em", "0.1em", "0.3em", "4px", "-1.5px"];

export function Case() {
  return (
    <div style={{ position: "absolute", left: 24, top: 12, right: 24 }}>
      {SPACINGS.map((letterSpacing) => (
        <div
          key={letterSpacing}
          style={{
            letterSpacing,
            fontSize: 26,
            lineHeight: 1.25,
            fontWeight: 600,
            whiteSpace: "nowrap",
          }}
        >
          LETTERS spaced {letterSpacing}
        </div>
      ))}
    </div>
  );
}
