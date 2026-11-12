import { spring } from "@opsis/runtime/spring.ts";
import { MAIN_FONT } from "#video/common/fonts.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function AnimatedLineChart({
  data,
  width,
  height,
  strokeColor,
  strokeWidth,
  gridColor,
  showDot,
  speed,
  startAtZero,
  finishAt,
  gridRows,
  showColumns,
  glow,
}: ElementSettings["animated-line-chart"]) {
  const frame = useAuthoredFrame() * speed;
  const { fps, durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;

  const padding = 60;
  const innerWidth = width - padding * 2;
  const innerHeight = height - padding * 2;

  const min = startAtZero ? Math.min(0, ...data) : Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;

  const points = data.map((value, index) => {
    const x = padding + (index / (data.length - 1)) * innerWidth;
    const y = padding + innerHeight - ((value - min) / range) * innerHeight;
    return { x, y };
  });

  let pathLength = 0;
  for (let i = 1; i < points.length; i++) {
    const dx = points[i].x - points[i - 1].x;
    const dy = points[i].y - points[i - 1].y;
    pathLength += Math.sqrt(dx * dx + dy * dy);
  }

  const d = points
    .map((p, i) => `${i === 0 ? "M" : "L"}${p.x.toFixed(2)},${p.y.toFixed(2)}`)
    .join(" ");

  const progress = spring({
    frame,
    fps,
    durationInFrames: Math.round((durationInFrames * finishAt) / 100),
    config: { damping: 200 },
  });

  const dashOffset = pathLength * (1 - progress);

  const targetLen = pathLength * progress;
  let traveled = 0;
  let dotX = points[0].x;
  let dotY = points[0].y;
  for (let i = 1; i < points.length; i++) {
    const dx = points[i].x - points[i - 1].x;
    const dy = points[i].y - points[i - 1].y;
    const segLen = Math.sqrt(dx * dx + dy * dy);
    if (traveled + segLen >= targetLen) {
      const t = (targetLen - traveled) / segLen;
      dotX = points[i - 1].x + dx * t;
      dotY = points[i - 1].y + dy * t;
      break;
    }
    traveled += segLen;
    dotX = points[i].x;
    dotY = points[i].y;
  }

  const gridCols = showColumns ? data.length - 1 : 0;

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: MAIN_FONT,
      }}
    >
      <svg
        width={width}
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        style={{ overflow: "visible" }}
      >
        {Array.from({ length: gridRows + 1 }).map((_, i) => {
          const y = padding + (i / gridRows) * innerHeight;
          return (
            <line
              key={`h-${i}`}
              x1={padding}
              x2={padding + innerWidth}
              y1={y}
              y2={y}
              stroke={gridColor}
              strokeWidth={1}
            />
          );
        })}
        {Array.from({ length: showColumns ? gridCols + 1 : 0 }).map((_, i) => {
          const x = padding + (i / gridCols) * innerWidth;
          return (
            <line
              key={`v-${i}`}
              x1={x}
              x2={x}
              y1={padding}
              y2={padding + innerHeight}
              stroke={gridColor}
              strokeWidth={1}
            />
          );
        })}
        <line
          x1={padding}
          x2={padding}
          y1={padding}
          y2={padding + innerHeight}
          stroke={gridColor}
          strokeWidth={2}
        />
        <line
          x1={padding}
          x2={padding + innerWidth}
          y1={padding + innerHeight}
          y2={padding + innerHeight}
          stroke={gridColor}
          strokeWidth={2}
        />

        <path
          d={d}
          fill="none"
          stroke={strokeColor}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeDasharray={pathLength}
          strokeDashoffset={dashOffset}
          style={{
            filter: `drop-shadow(0 0 12px color-mix(in srgb, ${strokeColor} ${glow}%, transparent))`,
          }}
        />

        {showDot && progress > 0 && progress < 1 && (
          <circle
            cx={dotX}
            cy={dotY}
            r={strokeWidth * 2}
            fill={strokeColor}
            style={{
              filter: `drop-shadow(0 0 8px ${strokeColor})`,
            }}
          />
        )}
      </svg>
    </div>
  );
}
