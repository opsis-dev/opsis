import React from "react";
import { useVideoConfig } from "remotion";
import type { Box } from "#video/core/still-layout.ts";
import { PixelateRegion } from "#video/elements/effect/mosaic/pixelate-region.tsx";

const MAX_REGIONS = 8;

export function Mosaic({
  boxes = [],
  blockSize = 24,
  children,
}: {
  boxes?: Box[];
  blockSize?: number;
  children?: React.ReactNode;
}) {
  const { width, height } = useVideoConfig();
  const regions = boxes
    .filter((b) => b.width > 0 && b.height > 0)
    .slice(0, MAX_REGIONS)
    .map((b) => ({
      x: b.x * width,
      y: b.y * height,
      width: b.width * width,
      height: b.height * height,
    }));
  if (regions.length === 0) return <>{children}</>;
  return (
    <PixelateRegion regions={regions} cellSize={blockSize}>
      {children}
    </PixelateRegion>
  );
}
