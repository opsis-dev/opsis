export type SnapTarget = {
  frame: number;
  kind: "playhead" | "edge" | "marker" | "action" | "beat" | "range";
};

export function snapFrame(
  frame: number,
  targets: SnapTarget[],
  threshold: number,
): { frame: number; target: SnapTarget | null } {
  let best: SnapTarget | null = null;
  let distance = threshold;
  for (const t of targets) {
    const d = Math.abs(t.frame - frame);
    if (d <= distance) {
      distance = d;
      best = t;
    }
  }
  return best ? { frame: best.frame, target: best } : { frame, target: null };
}

export function snapSpan(
  from: number,
  to: number,
  delta: number,
  targets: SnapTarget[],
  threshold: number,
): { delta: number; target: SnapTarget | null } {
  let best: { delta: number; target: SnapTarget; distance: number } | null = null;
  for (const edge of [from, to]) {
    const { frame, target } = snapFrame(edge + delta, targets, threshold);
    if (!target) continue;
    const distance = Math.abs(frame - (edge + delta));
    if (!best || distance < best.distance) best = { delta: frame - edge, target, distance };
  }
  return best ? { delta: best.delta, target: best.target } : { delta, target: null };
}

const FRAME_STEPS = [1, 2, 5, 10];
const FRAME_TICKS = 5;

const WIDEST_SECONDS = 600;
const SECOND_STEPS = [
  { seconds: 0.5, ticks: 2 },
  ...[1, 2, 5, 10, 15, 30, 60, 120, 300, WIDEST_SECONDS].map((seconds) => ({ seconds, ticks: 5 })),
];
const WIDEST_STEP = { seconds: WIDEST_SECONDS, ticks: 10 };
const MIN_STEP_PX = 72;

export function rulerStep(
  pxPerFrame: number,
  fps: number,
  minPx = MIN_STEP_PX,
): { step: number; minor: number } {
  for (const step of FRAME_STEPS)
    if (step * pxPerFrame >= minPx && step < fps / 2)
      return { step, minor: Math.max(1, step / FRAME_TICKS) };
  const fit =
    SECOND_STEPS.find(({ seconds }) => seconds * fps * pxPerFrame >= minPx) ?? WIDEST_STEP;
  const step = fit.seconds * fps;
  return { step, minor: step / fit.ticks };
}
