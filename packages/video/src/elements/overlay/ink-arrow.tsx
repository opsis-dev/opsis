import { AbsoluteFill, useVideoConfig } from "remotion";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { hashRange, steppedRamp } from "#video/lib/motion/stop-motion.ts";
import {
  BrushGrain,
  type BrushPoint,
  brushFilterId,
  brushReach,
  brushRibbon,
  sampleCubic,
  sampleLine,
} from "#video/parts/brush.tsx";

const SHAFT_SAMPLES = 40;
const ARM_SAMPLES = 10;
const ARM_TAPER = 0.6;
const ARM_OVERSHOOT = 0.16;
const HEAD_MIN_RATIO = 2.6;

export type InkArrowPoint = BrushPoint;

export function inkArrowControls(
  from: InkArrowPoint,
  to: InkArrowPoint,
  curvature: number,
  seed: string,
  wobbleShare: number,
): { c1: InkArrowPoint; c2: InkArrowPoint } {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const distance = Math.hypot(dx, dy) || 1;
  const normalX = -dy / distance;
  const normalY = dx / distance;
  const bow = curvature * distance;
  const wobble = distance * wobbleShare;

  const at = (t: number, key: string): InkArrowPoint => ({
    x: from.x + dx * t + normalX * bow + hashRange(`${seed}:${key}:x`, -wobble, wobble),
    y: from.y + dy * t + normalY * bow + hashRange(`${seed}:${key}:y`, -wobble, wobble),
  });

  return { c1: at(0.3, "c1"), c2: at(0.7, "c2") };
}

export function inkArrowHead(
  c2: InkArrowPoint,
  to: InkArrowPoint,
  headSize: number,
  headAngle: number,
): { left: InkArrowPoint; right: InkArrowPoint } {
  const dx = to.x - c2.x;
  const dy = to.y - c2.y;
  const length = Math.hypot(dx, dy) || 1;
  const backX = -dx / length;
  const backY = -dy / length;

  const arm = (angle: number): InkArrowPoint => ({
    x: to.x + headSize * (backX * Math.cos(angle) - backY * Math.sin(angle)),
    y: to.y + headSize * (backX * Math.sin(angle) + backY * Math.cos(angle)),
  });

  return { left: arm(headAngle), right: arm(-headAngle) };
}

export function inkArrowArmWidth(strokeWidth: number, release: number): number {
  return strokeWidth * release;
}

export function inkArrowApex(c2: InkArrowPoint, to: InkArrowPoint, reach: number): InkArrowPoint {
  const dx = to.x - c2.x;
  const dy = to.y - c2.y;
  const length = Math.hypot(dx, dy) || 1;
  return {
    x: to.x + (dx / length) * reach * ARM_OVERSHOOT,
    y: to.y + (dy / length) * reach * ARM_OVERSHOOT,
  };
}

export function inkArrowHeadReach(headSize: number, strokeWidth: number): number {
  return Math.max(headSize, strokeWidth * HEAD_MIN_RATIO);
}

export function inkArrowViewport(
  from: InkArrowPoint,
  to: InkArrowPoint,
  curvature: number,
  headSize: number,
  strokeWidth: number,
  grain = 1,
): { width: number; height: number } {
  const distance = Math.hypot(to.x - from.x, to.y - from.y);
  const margin =
    inkArrowHeadReach(headSize, strokeWidth) +
    Math.abs(curvature) * distance +
    brushReach(strokeWidth, grain);
  return {
    width: Math.max(margin, Math.max(from.x, to.x) + margin),
    height: Math.max(margin, Math.max(from.y, to.y) + margin),
  };
}

type InkArrowProps = Omit<ElementSettings["ink-arrow"], "fromX" | "fromY" | "toX" | "toY"> & {
  from: InkArrowPoint;
  to: InkArrowPoint;
};

function InkArrowView({
  from,
  to,
  curvature,
  color,
  strokeWidth,
  pressure,
  release,
  grain,
  drawDur,
  headSize,
  headDur,
  seed,
  step,
  headAngle,
  wobble,
  opacity,
}: InkArrowProps) {
  const frame = useAuthoredFrame();
  const progress = steppedRamp(frame, 0, drawDur, { step });
  const { c1, c2 } = inkArrowControls(from, to, curvature, seed, wobble);
  const headReach = inkArrowHeadReach(headSize, strokeWidth);
  const head = inkArrowHead(c2, to, headReach, (headAngle * Math.PI) / 180);
  const apex = inkArrowApex(c2, to, headReach);

  const headFrames = headDur;
  const headStart = drawDur;
  const headMid = headStart + headFrames / 2;
  const leftArm = steppedRamp(frame, headStart, headMid, { step });
  const rightArm = steppedRamp(frame, headMid, headStart + headFrames, {
    step,
  });

  const { width, height } = inkArrowViewport(from, to, curvature, headSize, strokeWidth, grain);

  const filterId = brushFilterId(seed, strokeWidth, grain);
  const filter = grain > 0 ? `url(#${filterId})` : undefined;

  const armRibbon = (tip: InkArrowPoint, armProgress: number) =>
    brushRibbon(sampleLine(apex, tip, ARM_SAMPLES), {
      strokeWidth: inkArrowArmWidth(strokeWidth, release),
      pressure: 1,
      release: ARM_TAPER,
      progress: armProgress,
    });

  return (
    <svg
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      style={{
        display: "block",
        overflow: "visible",
        opacity: progress > 0 ? 1 : 0,
      }}
    >
      <title>Ink arrow</title>
      <BrushGrain seed={seed} strokeWidth={strokeWidth} grain={grain} />
      <path
        d={brushRibbon(sampleCubic(from, c1, c2, to, SHAFT_SAMPLES), {
          strokeWidth,
          pressure,
          release,
          progress,
        })}
        fill={color}
        opacity={opacity}
        filter={filter}
      />
      {leftArm > 0 ? (
        <path d={armRibbon(head.left, leftArm)} fill={color} opacity={opacity} filter={filter} />
      ) : null}
      {rightArm > 0 ? (
        <path d={armRibbon(head.right, rightArm)} fill={color} opacity={opacity} filter={filter} />
      ) : null}
    </svg>
  );
}

export function InkArrow({ fromX, fromY, toX, toY, ...settings }: ElementSettings["ink-arrow"]) {
  const { width, height } = useVideoConfig();
  const point = (x: number, y: number) => ({ x: x * width, y: y * height });
  return (
    <AbsoluteFill>
      <InkArrowView {...settings} from={point(fromX, fromY)} to={point(toX, toY)} />
    </AbsoluteFill>
  );
}
