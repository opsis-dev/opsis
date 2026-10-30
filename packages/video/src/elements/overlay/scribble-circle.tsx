import { AbsoluteFill, useVideoConfig } from "remotion";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { randomBetween, steppedRamp } from "#video/lib/motion/stop-motion.ts";
import {
  BrushGrain,
  type BrushPoint,
  brushFilterId,
  brushReach,
  brushRibbon,
} from "#video/parts/brush.tsx";

const SAMPLES_PER_LAP = 48;
const CENTRE = 0.02;

type ScribbleCircleGeometry = {
  width: number;
  height: number;
  strokeWidth: number;
  laps: number;
  pressure: number;
  release: number;
  grain: number;
  seed: string;
  progress: number;
  wobble: number;
  startAngle: number;
  clockwise: boolean;
};

function scribbleCircleCentre(width: number, height: number, seed: string): BrushPoint {
  return {
    x: width / 2 + randomBetween(`${seed}:cx`, -CENTRE, CENTRE) * width,
    y: height / 2 + randomBetween(`${seed}:cy`, -CENTRE, CENTRE) * height,
  };
}

function scribbleCirclePoints(args: ScribbleCircleGeometry): BrushPoint[] {
  const { laps, seed, wobble } = args;
  const count = Math.max(2, Math.ceil(laps * SAMPLES_PER_LAP));
  const rx = args.width / 2;
  const ry = args.height / 2;
  const { x: cx, y: cy } = scribbleCircleCentre(args.width, args.height, seed);
  const start = (args.startAngle * Math.PI) / 180;
  const turn = args.clockwise ? 1 : -1;

  return Array.from({ length: count }, (_, i) => {
    const angle = start + turn * laps * Math.PI * 2 * (i / (count - 1));
    const scale = 1 + randomBetween(`${seed}:r${i}`, -wobble, wobble);
    return {
      x: cx + rx * scale * Math.cos(angle),
      y: cy + ry * scale * Math.sin(angle),
    };
  });
}

function scribbleCirclePath(args: ScribbleCircleGeometry): {
  d: string;
  viewBox: string;
  marginX: number;
  marginY: number;
} {
  const reach = brushReach(args.strokeWidth, args.grain);
  const marginX = reach + (args.width / 2) * args.wobble + args.width * CENTRE;
  const marginY = reach + (args.height / 2) * args.wobble + args.height * CENTRE;

  return {
    d: brushRibbon(scribbleCirclePoints(args), {
      strokeWidth: args.strokeWidth,
      pressure: args.pressure,
      release: args.release,
      progress: args.progress,
    }),
    viewBox: `${-marginX} ${-marginY} ${args.width + marginX * 2} ${args.height + marginY * 2}`,
    marginX,
    marginY,
  };
}

function ScribbleCircleView({
  width,
  height,
  color,
  strokeWidth,
  pressure,
  release,
  grain,
  durationSteps,
  laps,
  seed,
  step,
  wobble,
  startAngle,
  clockwise,
  opacity,
}: Omit<ElementSettings["scribble-circle"], "x" | "y">) {
  const frame = useAuthoredFrame();
  const progress = steppedRamp(frame, 0, durationSteps * step, { step });
  const { d, viewBox, marginX, marginY } = scribbleCirclePath({
    width,
    height,
    strokeWidth,
    laps,
    pressure,
    release,
    grain,
    seed,
    progress,
    wobble,
    startAngle,
    clockwise,
  });

  return (
    <div
      style={{
        position: "relative",
        width,
        height,
        opacity: progress > 0 ? 1 : 0,
      }}
    >
      <svg
        width={width + marginX * 2}
        height={height + marginY * 2}
        viewBox={viewBox}
        style={{
          position: "absolute",
          left: -marginX,
          top: -marginY,
          display: "block",
          overflow: "visible",
        }}
      >
        <title>Scribbled circle</title>
        <BrushGrain seed={seed} strokeWidth={strokeWidth} grain={grain} />
        <path
          d={d}
          fill={color}
          opacity={opacity}
          filter={grain > 0 ? `url(#${brushFilterId(seed, strokeWidth, grain)})` : undefined}
        />
      </svg>
    </div>
  );
}

export function ScribbleCircle({ x, y, ...settings }: ElementSettings["scribble-circle"]) {
  const { width, height } = useVideoConfig();
  return (
    <AbsoluteFill>
      <div
        style={{
          position: "absolute",
          left: x * width,
          top: y * height,
          translate: "-50% -50%",
        }}
      >
        <ScribbleCircleView {...settings} />
      </div>
    </AbsoluteFill>
  );
}
