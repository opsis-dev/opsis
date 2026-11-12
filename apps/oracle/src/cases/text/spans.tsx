const WORDS = ["Every", "word", "moves", "on", "its", "own"];

export function Case() {
  return (
    <div
      style={{ position: "absolute", left: 24, top: 24, right: 24, fontSize: 26, lineHeight: 1.4 }}
    >
      <p>
        Plain text with <strong>strong</strong>, <em>emphasis</em>,{" "}
        <span style={{ color: "#f97316", fontWeight: 700 }}>a coloured run</span>,{" "}
        <span
          style={{
            fontFamily: "JetBrains Mono",
            fontSize: "0.8em",
            background: "#334155",
            padding: "0 4px",
            borderRadius: 4,
          }}
        >
          code
        </span>{" "}
        and <span style={{ fontSize: "1.4em", fontFamily: "Fraunces" }}>bigger</span> words on one
        line that wraps.
      </p>
      <p style={{ marginTop: 16 }}>
        {WORDS.map((word, i) => (
          <span
            key={word}
            style={{
              display: "inline-block",
              marginRight: "0.25em",
              translate: `0 ${(i % 3) * 6}px`,
              opacity: 1 - i * 0.12,
              rotate: `${i * 2 - 5}deg`,
            }}
          >
            {word}
          </span>
        ))}
      </p>
      <p style={{ marginTop: 8, fontSize: 18 }}>
        H<sub>2</sub>O and E = mc<sup>2</sup>, <u>underlined</u>, <s>struck</s>,{" "}
        <span style={{ textDecoration: "underline wavy #f43f5e" }}>wavy</span>
      </p>
    </div>
  );
}
