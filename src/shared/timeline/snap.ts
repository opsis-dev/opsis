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

export function formatTime(frame: number, fps: number, mode: "seconds" | "timecode"): string {
  const f = Math.max(0, Math.round(frame));
  if (mode === "timecode") {
    const total = Math.floor(f / fps);
    const ff = f % fps;
    const ss = total % 60;
    const mm = Math.floor(total / 60) % 60;
    const hh = Math.floor(total / 3600);
    const p = (n: number) => String(n).padStart(2, "0");
    return `${p(hh)}:${p(mm)}:${p(ss)}:${p(ff)}`;
  }
  const secs = f / fps;
  const m = Math.floor(secs / 60);
  const s = secs - m * 60;
  const text = s.toFixed(s % 1 === 0 ? 0 : 1);
  return `${m}:${text.padStart(text.includes(".") ? 4 : 2, "0")}`;
}

const STEPS_FRAMES = [1, 2, 5, 10];
const STEPS_SECONDS = [0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600];

export function rulerStep(
  pxPerFrame: number,
  fps: number,
  minPx = 72,
): { step: number; minor: number } {
  for (const s of STEPS_FRAMES)
    if (s * pxPerFrame >= minPx && s < fps / 2) return { step: s, minor: Math.max(1, s / 5) };
  for (const sec of STEPS_SECONDS) {
    const step = sec * fps;
    if (step * pxPerFrame >= minPx) return { step, minor: step / (sec >= 1 ? 5 : 2) };
  }
  return { step: 600 * fps, minor: 60 * fps };
}
