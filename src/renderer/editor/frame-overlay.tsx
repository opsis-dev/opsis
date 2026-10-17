import type { ReactNode } from "react";

type Box = { x: number; y: number; width: number; height: number };

const BOX_RADIUS = 6;
const BOX_STROKE = 1 / 480;
const POINT_STROKE = 1 / 960;

export function FrameOverlay({
  width,
  height,
  box,
  point,
  pointSize,
  children,
}: {
  width: number;
  height: number;
  box?: Box | null;
  point?: { x: number; y: number } | null;
  pointSize: number;
  children?: ReactNode;
}) {
  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      className="pointer-events-none absolute inset-0 size-full"
    >
      {box ? (
        <rect
          x={box.x}
          y={box.y}
          width={box.width}
          height={box.height}
          rx={BOX_RADIUS}
          className="fill-primary/10 stroke-primary"
          strokeWidth={width * BOX_STROKE}
        />
      ) : null}
      {children}
      {point ? (
        <circle
          cx={point.x}
          cy={point.y}
          r={width * pointSize}
          className="fill-primary stroke-background"
          strokeWidth={width * POINT_STROKE}
        />
      ) : null}
    </svg>
  );
}
