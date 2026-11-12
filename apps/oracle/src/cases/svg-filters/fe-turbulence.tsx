import { Filtered } from "#oracle/filter-parts.tsx";

const NOISES: [string, string, number, number, boolean][] = [
  ["fractalNoise", "0.05", 1, 0, false],
  ["fractalNoise", "0.05", 4, 0, false],
  ["fractalNoise", "0.9", 3, 7, false],
  ["turbulence", "0.02", 2, 0, false],
  ["turbulence", "0.08 0.02", 3, 2, false],
  ["fractalNoise", "0.03", 5, 11, true],
  ["fractalNoise", "0.65", 1, 3, false],
  ["turbulence", "0.01", 1, 4, true],
];

export function Case() {
  return (
    <Filtered
      filters={NOISES.map(([type, baseFrequency, numOctaves, seed, stitch], i) => [
        `oracle-noise-${i}`,
        <feTurbulence
          key="noise"
          type={type as "fractalNoise"}
          baseFrequency={baseFrequency}
          numOctaves={numOctaves}
          seed={seed}
          stitchTiles={stitch ? "stitch" : "noStitch"}
        />,
      ])}
    />
  );
}
