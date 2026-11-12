import { Filtered } from "#oracle/filter-parts.tsx";

const OFFSETS: [number, number][] = [
  [0, 0],
  [6, 6],
  [-10, 4],
  [0, 20],
  [2.5, 1.5],
  [30, -10],
  [-4, -4],
  [12, 0],
];

export function Case() {
  return (
    <Filtered
      filters={OFFSETS.map(([dx, dy], i) => [
        `oracle-offset-${i}`,
        <>
          <feOffset key="offset" in="SourceAlpha" dx={dx} dy={dy} result="moved" />
          <feFlood key="flood" floodColor="#7c3aed" floodOpacity="0.8" result="tint" />
          <feComposite key="tinted" in="tint" in2="moved" operator="in" result="shadow" />
          <feMerge key="merge">
            <feMergeNode in="shadow" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </>,
      ])}
    />
  );
}
