import { SLATE, cell } from "#oracle/parts.tsx";

const TRANSFORMS = [
  "none",
  "translate(10px, 20px) rotate(20deg) scale(0.8)",
  "scale(0.8) rotate(20deg) translate(10px, 20px)",
  "skewX(20deg)",
  "skew(-10deg, 15deg)",
  "matrix(0.9, 0.3, -0.3, 0.9, 10, 0)",
  "translate(-50%, -50%) rotate(-12deg)",
  "rotate(45deg) scaleX(1.5)",
];

export function Case() {
  return (
    <>
      {TRANSFORMS.map((transform, i) => (
        <div key={i} style={cell(i, { background: SLATE })}>
          <div
            style={{
              position: "absolute",
              left: i === 6 ? "50%" : 30,
              top: i === 6 ? "50%" : 40,
              width: 70,
              height: 50,
              background: "linear-gradient(135deg, #f43f5e, #facc15)",
              transform,
            }}
          />
        </div>
      ))}
    </>
  );
}
