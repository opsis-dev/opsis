import type { Bounds } from "@opsis/video/common/geometry";
import type { ReactNode } from "react";

const BOX_RADIUS = 6;

export const OVERLAY_WIDTHS = {
  boxStroke: 1 / 480,
  pointStroke: 1 / 960,
  rippleStroke: 1 / 640,
  livePoint: 1 / 220,
  tracePoint: 1 / 160,
} as const;

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
  box?: Bounds | null;
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
          strokeWidth={width * OVERLAY_WIDTHS.boxStroke}
        />
      ) : null}
      {children}
      {point ? (
        <circle
          cx={point.x}
          cy={point.y}
          r={width * pointSize}
          className="fill-primary stroke-background"
          strokeWidth={width * OVERLAY_WIDTHS.pointStroke}
        />
      ) : null}
    </svg>
  );
}
