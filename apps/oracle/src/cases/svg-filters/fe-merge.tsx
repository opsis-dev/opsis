import { Filtered } from "#oracle/filter-parts.tsx";

export function Case() {
  return (
    <Filtered
      filters={[
        [
          "oracle-merge-glow",
          <>
            <feGaussianBlur key="blur" in="SourceGraphic" stdDeviation="4" result="glow" />
            <feMerge key="merge">
              <feMergeNode in="glow" />
              <feMergeNode in="glow" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </>,
        ],
        [
          "oracle-merge-shadow",
          <>
            <feGaussianBlur key="blur" in="SourceAlpha" stdDeviation="3" result="soft" />
            <feOffset key="offset" in="soft" dx="4" dy="6" result="drop" />
            <feMerge key="merge">
              <feMergeNode in="drop" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </>,
        ],
        [
          "oracle-merge-copies",
          <>
            <feOffset key="a" in="SourceGraphic" dx="-6" result="left" />
            <feOffset key="b" in="SourceGraphic" dx="6" result="right" />
            <feColorMatrix
              key="c"
              in="left"
              type="matrix"
              values="1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 0.7 0"
              result="red"
            />
            <feColorMatrix
              key="d"
              in="right"
              type="matrix"
              values="0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 0.7 0"
              result="blue"
            />
            <feMerge key="merge">
              <feMergeNode in="red" />
              <feMergeNode in="blue" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </>,
        ],
        [
          "oracle-drop-shadow",
          <feDropShadow
            key="drop"
            dx="3"
            dy="5"
            stdDeviation="2"
            floodColor="#000"
            floodOpacity="0.8"
          />,
        ],
      ]}
    />
  );
}
