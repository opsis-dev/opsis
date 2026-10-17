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
  const a = snapFrame(from + delta, targets, threshold);
  const b = snapFrame(to + delta, targets, threshold);
  const da = a.target ? Math.abs(a.frame - (from + delta)) : Infinity;
  const db = b.target ? Math.abs(b.frame - (to + delta)) : Infinity;
  if (da === Infinity && db === Infinity) return { delta, target: null };
  return da <= db
    ? { delta: a.frame - from, target: a.target }
    : { delta: b.frame - to, target: b.target };
}

const FRAME_STEPS = [1, 2, 5, 10];
const FRAME_TICKS = 5;

const SECOND_STEPS = [
  { seconds: 0.5, ticks: 2 },
  ...[1, 2, 5, 10, 15, 30, 60, 120, 300, 600].map((seconds) => ({ seconds, ticks: 5 })),
];
const WIDEST_STEP = { seconds: 600, ticks: 10 };

export function rulerStep(
  pxPerFrame: number,
  fps: number,
  minPx = 72,
): { step: number; minor: number } {
  for (const step of FRAME_STEPS)
    if (step * pxPerFrame >= minPx && step < fps / 2)
      return { step, minor: Math.max(1, step / FRAME_TICKS) };
  const fit =
    SECOND_STEPS.find(({ seconds }) => seconds * fps * pxPerFrame >= minPx) ?? WIDEST_STEP;
  const step = fit.seconds * fps;
  return { step, minor: step / fit.ticks };
}
