const SIZES = [8, 11, 13.5, 16, 24, 36, 64, 120];

export function Case() {
  return (
    <div style={{ position: "absolute", left: 24, top: 8, right: 24, lineHeight: 1 }}>
      {SIZES.map((fontSize) => (
        <div key={fontSize} style={{ fontSize, whiteSpace: "nowrap" }}>
          Ag {fontSize}px
        </div>
      ))}
    </div>
  );
}
