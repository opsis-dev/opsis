import { Filtered } from "#oracle/filter-parts.tsx";

const MODES = [
  "normal",
  "multiply",
  "screen",
  "overlay",
  "darken",
  "lighten",
  "difference",
  "color-dodge",
];

export function Case() {
  return (
    <Filtered
      filters={MODES.map((mode, i) => [
        `oracle-blend-${i}`,
        <>
          <feFlood key="flood" floodColor="#22c55e" floodOpacity="0.7" result="wash" />
          <feBlend key="blend" in="SourceGraphic" in2="wash" mode={mode as "normal"} />
        </>,
      ])}
    />
  );
}
