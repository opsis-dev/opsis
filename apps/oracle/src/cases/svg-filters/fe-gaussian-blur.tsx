import { Filtered } from "#oracle/filter-parts.tsx";

const DEVIATIONS = ["0", "0.5", "1", "2", "4", "8", "3 0", "0 6"];

export function Case() {
  return (
    <Filtered
      filters={DEVIATIONS.map((stdDeviation, i) => [
        `oracle-blur-${i}`,
        <feGaussianBlur key="blur" in="SourceGraphic" stdDeviation={stdDeviation} />,
      ])}
    />
  );
}
