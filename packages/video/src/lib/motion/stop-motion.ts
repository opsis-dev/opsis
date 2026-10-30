import { random, spring } from "remotion";

export const DEFAULT_STEP = 3;

export function qf(frame: number, step = DEFAULT_STEP): number {
  return Math.floor(frame / step) * step;
}

export function qstep(frame: number, step = DEFAULT_STEP): number {
  return Math.floor(frame / step);
}

export function randomBetween(seed: string, lo: number, hi: number): number {
  return lo + random(seed) * (hi - lo);
}

export type Jitter = { x: number; y: number; rot: number };

export function paperJitter(
  frame: number,
  seed: string,
  options?: { amp?: number; rotAmp?: number; step?: number },
): Jitter {
  const amp = options?.amp ?? 1.4;
  const rotAmp = options?.rotAmp ?? 0.35;
  const s = qstep(frame, options?.step ?? DEFAULT_STEP);
  return {
    x: randomBetween(`${seed}:x:${s}`, -amp, amp),
    y: randomBetween(`${seed}:y:${s}`, -amp, amp),
    rot: randomBetween(`${seed}:r:${s}`, -rotAmp, rotAmp),
  };
}

export function steppedSpring(args: {
  frame: number;
  fps: number;
  delay?: number;
  step?: number;
  config?: { damping?: number; stiffness?: number; mass?: number };
}): number {
  return spring({
    frame: qf(Math.max(0, args.frame - (args.delay ?? 0)), args.step ?? DEFAULT_STEP),
    fps: args.fps,
    config: args.config,
  });
}

export function steppedRamp(
  frame: number,
  from: number,
  to: number,
  options?: { ease?: (t: number) => number; step?: number },
): number {
  const f = qf(frame, options?.step ?? DEFAULT_STEP);
  if (f <= from) return 0;
  if (f >= to) return 1;
  const ease = options?.ease ?? ((t: number) => t);
  return ease((f - from) / (to - from));
}
