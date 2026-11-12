import { Filtered } from "#oracle/filter-parts.tsx";

const MATRICES: [string, string | undefined][] = [
  ["saturate", "0"],
  ["saturate", "2"],
  ["hueRotate", "90"],
  ["hueRotate", "200"],
  ["luminanceToAlpha", undefined],
  ["matrix", "0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0 0 0 1 0"],
  ["matrix", "1 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 0.6 0"],
  ["matrix", "0 0 0 0 1  0 0 0 0 0.8  0 0 0 0 0.2  0 0 0 1 0"],
];

export function Case() {
  return (
    <Filtered
      filters={MATRICES.map(([type, values], i) => [
        `oracle-matrix-${i}`,
        <feColorMatrix key="matrix" type={type as "matrix"} values={values} />,
      ])}
    />
  );
}
