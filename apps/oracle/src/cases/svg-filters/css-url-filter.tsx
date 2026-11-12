import { SAMPLE, cell } from "#oracle/parts.tsx";

export function Case() {
  return (
    <>
      <svg width="0" height="0" style={{ position: "absolute" }}>
        <defs>
          <filter id="oracle-rough">
            <feTurbulence
              type="fractalNoise"
              baseFrequency="0.04"
              numOctaves="3"
              seed="2"
              result="noise"
            />
            <feDisplacementMap
              in="SourceGraphic"
              in2="noise"
              scale="8"
              xChannelSelector="R"
              yChannelSelector="G"
            />
          </filter>
          <filter id="oracle-grain">
            <feTurbulence
              type="fractalNoise"
              baseFrequency="0.8"
              numOctaves="2"
              seed="5"
              result="grain"
            />
            <feColorMatrix in="grain" type="saturate" values="0" result="grey" />
            <feBlend in="SourceGraphic" in2="grey" mode="overlay" />
          </filter>
          <filter id="oracle-split">
            <feOffset in="SourceGraphic" dx="-3" result="left" />
            <feColorMatrix
              in="left"
              type="matrix"
              values="1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0"
              result="red"
            />
            <feColorMatrix
              in="SourceGraphic"
              type="matrix"
              values="0 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 1 0"
              result="rest"
            />
            <feBlend in="red" in2="rest" mode="screen" />
          </filter>
        </defs>
      </svg>
      <div
        style={cell(0, { filter: "url(#oracle-rough)", background: "#f97316", borderRadius: 24 })}
      />
      <div
        style={cell(1, {
          filter: "url(#oracle-grain)",
          background: "linear-gradient(#0ea5e9, #7c3aed)",
        })}
      />
      <div
        style={cell(2, {
          filter: "url(#oracle-split)",
          fontSize: 28,
          fontWeight: 900,
          lineHeight: 1.1,
        })}
      >
        {SAMPLE}
      </div>
      <div
        style={cell(3, {
          filter: "url(#oracle-rough) drop-shadow(0 4px 4px #000)",
          fontSize: 28,
          fontWeight: 900,
          color: "#22c55e",
        })}
      >
        Wobbly
      </div>
    </>
  );
}
