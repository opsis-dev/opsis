import { SLATE, cell } from "#oracle/parts.tsx";

const MATRICES = [
  "matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1)",
  "matrix3d(0.9,0.1,0,0.001, -0.1,0.9,0,0.0005, 0,0,1,0, 10,5,0,1)",
  "matrix3d(1,0,0,0.002, 0,1,0,0, 0,0,1,0, 0,0,0,1)",
  "matrix3d(1,0,0,0, 0,1,0,0.003, 0,0,1,0, 0,0,0,1)",
  "matrix3d(0.8,0.2,0,-0.0015, 0.1,1,0,0.001, 0,0,1,0, 0,0,0,1)",
  "matrix3d(1.2,0,0,0.0025, 0,0.8,0,-0.001, 0,0,1,0, -10,10,0,1)",
  "matrix3d(0.7,-0.3,0,0, 0.3,0.7,0,0.002, 0,0,1,0, 20,0,0,1.1)",
  "matrix3d(1,0.4,0,0.004, 0,1,0,0.004, 0,0,1,0, 0,0,0,1)",
];

export function Case() {
  return (
    <>
      {MATRICES.map((transform, i) => (
        <div key={i} style={cell(i, { background: SLATE })}>
          <div
            style={{
              position: "absolute",
              left: 0,
              top: 0,
              width: 100,
              height: 100,
              transformOrigin: "0 0",
              transform,
              background:
                "repeating-linear-gradient(90deg, #f97316 0 10px, #0f172a 10px 20px), #f97316",
              border: "2px solid #f8fafc",
            }}
          />
        </div>
      ))}
    </>
  );
}
