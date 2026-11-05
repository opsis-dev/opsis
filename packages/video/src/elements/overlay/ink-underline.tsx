import { useFilm } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { randomBetween, steppedRamp } from "#video/lib/motion/stop-motion.ts";
import {
  BrushGrain,
  type BrushPoint,
  brushFilterId,
  brushReach,
  brushRibbon,
  sampleCubic,
} from "#video/parts/brush.tsx";

const SAMPLES = 40;

function inkUnderlineSpine(
  width: number,
  thickness: number,
  seed: string,
  wobbleSize: number,
): BrushPoint[] {
  const wobble = randomBetween(`${seed}:w`, -wobbleSize, wobbleSize);
  const mid = thickness / 2 + 2;
  return sampleCubic(
    { x: 2, y: mid },
    { x: width * 0.3, y: mid + wobble },
    { x: width * 0.7, y: mid - wobble },
    { x: width - 2, y: mid + wobble * 0.6 },
    SAMPLES,
  );
}

function InkUnderlineView({
  width,
  color,
  thickness,
  pressure,
  release,
  grain,
  durationSteps,
  seed,
  step,
  wobble,
  opacity,
}: Omit<ElementSettings["ink-underline"], "x" | "y" | "angle">) {
  const frame = useAuthoredFrame();
  const progress = steppedRamp(frame, 0, durationSteps * step, {
    step,
  });

  const reach = brushReach(thickness, grain);
  const height = thickness + 4;
  const d = brushRibbon(inkUnderlineSpine(width, thickness, seed, wobble), {
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
          opacity={opacity}
          filter={grain > 0 ? `url(#${brushFilterId(seed, thickness, grain)})` : undefined}
        />
      </svg>
    </div>
  );
}

export function InkUnderline({ x, y, angle, ...settings }: ElementSettings["ink-underline"]) {
  const { width, height } = useFilm();
  return (
    <AbsoluteFill>
      <div
        style={{
          position: "absolute",
          left: x * width,
          top: y * height,
          translate: "-50% -50%",
          rotate: `${angle}deg`,
        }}
      >
        <InkUnderlineView {...settings} />
      </div>
    </AbsoluteFill>
  );
}
