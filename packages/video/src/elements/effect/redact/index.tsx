import { useFilm } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type React from "react";
import { PixelateRegion } from "#video/elements/effect/redact/pixelate-region.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

const MAX_REGIONS = 8;

type Region = { x: number; y: number; width: number; height: number };

function Covers({
  regions,
  style,
}: {
  regions: Region[];
  style: (region: Region) => React.CSSProperties;
}) {
  return (
    <AbsoluteFill style={{ pointerEvents: "none" }}>
      {regions.map((region) => (
        <div
          key={`${region.x}:${region.y}:${region.width}:${region.height}`}
          style={{
            position: "absolute",
            left: region.x,
            top: region.y,
            width: region.width,
            height: region.height,
            ...style(region),
          }}
        />
      ))}
    </AbsoluteFill>
  );
}

export function Redact({
  boxes,
  style,
  blockSize,
  blur,
  color,
  radius,
  children,
}: ElementSettings["redact"] & { children?: React.ReactNode }) {
  const { width, height } = useFilm();
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
  if (style === "pixelate") {
    return (
      <PixelateRegion regions={regions} cellSize={blockSize}>
        {children}
      </PixelateRegion>
    );
  }
  return (
    <AbsoluteFill>
      {children}
      <Covers
        regions={regions}
        style={() =>
          style === "blur"
            ? { backdropFilter: `blur(${blur}px)`, WebkitBackdropFilter: `blur(${blur}px)` }
            : { background: color, borderRadius: radius }
        }
      />
    </AbsoluteFill>
  );
}
