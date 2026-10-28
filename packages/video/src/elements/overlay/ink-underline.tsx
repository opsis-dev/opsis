import type React from "react";
import { AbsoluteFill, useVideoConfig } from "remotion";
import { numberOr } from "#video/common/math.ts";
import { type ElementProps } from "#video/common/spec.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import { DEFAULT_STEP, hashRange, steppedRamp } from "#video/lib/motion/stop-motion.ts";
import {
  BrushGrain,
  type BrushPoint,
  brushFilterId,
  brushReach,
  brushRibbon,
  sampleCubic,
} from "#video/parts/brush.tsx";

const SAMPLES = 40;

export function inkUnderlineSpine(width: number, thickness: number, seed: string): BrushPoint[] {
  const wobble = hashRange(`${seed}:w`, -1.2, 1.2);
  const mid = thickness / 2 + 2;
  return sampleCubic(
    { x: 2, y: mid },
    { x: width * 0.3, y: mid + wobble },
    { x: width * 0.7, y: mid - wobble },
    { x: width - 2, y: mid + wobble * 0.6 },
    SAMPLES,
  );
}

export interface InkUnderlineProps {
  width: number;
  color?: string;
  thickness?: number;
  pressure?: number;
  release?: number;
  grain?: number;
  delay?: number;
  durationSteps?: number;
  seed?: string;
  step?: number;
}

function InkUnderlineView({
  width,
  color = "#6f7f35",
  thickness = 9,
  pressure = 1,
  release = 0.15,
  grain = 1,
  delay = 0,
  durationSteps = 5,
  seed = "ink",
  step = DEFAULT_STEP,
}: InkUnderlineProps) {
  const frame = useAuthoredFrame();
  const progress = steppedRamp(frame, delay, delay + durationSteps * step, {
    step,
  });

  const reach = brushReach(thickness, grain);
  const height = thickness + 4;
  const d = brushRibbon(inkUnderlineSpine(width, thickness, seed), {
    strokeWidth: thickness,
    pressure,
    release,
    progress,
  });

  return (
    <div
      style={{
        width,
        height,
        position: "relative",
        opacity: progress > 0 ? 1 : 0,
      }}
    >
      <svg
        width={width + reach * 2}
        height={height + reach * 2}
        viewBox={`${-reach} ${-reach} ${width + reach * 2} ${height + reach * 2}`}
        style={{
          position: "absolute",
          left: -reach,
          top: -reach,
          display: "block",
          overflow: "visible",
        }}
      >
        <title>Ink underline</title>
        <BrushGrain seed={seed} strokeWidth={thickness} grain={grain} />
        <path
          d={d}
          fill={color}
          opacity={0.85}
          filter={grain > 0 ? `url(#${brushFilterId(seed, thickness, grain)})` : undefined}
        />
      </svg>
    </div>
  );
}

export function InkUnderline(all: ElementProps & { children?: React.ReactNode }) {
  const { x, y, ...props } = all;
  const { width, height } = useVideoConfig();
  return (
    <AbsoluteFill>
      <div
        style={{
          position: "absolute",
          left: numberOr(x, 0.5) * width,
          top: numberOr(y, 0.5) * height,
          translate: "-50% -50%",
        }}
      >
        <InkUnderlineView
          {...(props as unknown as React.ComponentProps<typeof InkUnderlineView>)}
        />
      </div>
    </AbsoluteFill>
  );
}
