import { Filtered } from "#oracle/filter-parts.tsx";

const MAPS: [string, number, string, string][] = [
  ["0.02", 0, "R", "G"],
  ["0.02", 10, "R", "G"],
  ["0.02", 30, "R", "G"],
  ["0.05", 20, "G", "B"],
  ["0.2", 8, "R", "A"],
  ["0.01 0.1", 25, "B", "R"],
  ["0.04", -20, "R", "G"],
  ["0.9", 6, "A", "A"],
];

export function Case() {
  return (
    <Filtered
      filters={MAPS.map(([baseFrequency, scale, xChannel, yChannel], i) => [
        `oracle-displace-${i}`,
        <>
          <feTurbulence
            key="noise"
            type="fractalNoise"
            baseFrequency={baseFrequency}
            numOctaves={2}
            seed={3}
            result="noise"
          />
          <feDisplacementMap
            key="displace"
            in="SourceGraphic"
            in2="noise"
            scale={scale}
            xChannelSelector={xChannel as "R"}
            yChannelSelector={yChannel as "G"}
          />
        </>,
      ])}
    />
  );
}
