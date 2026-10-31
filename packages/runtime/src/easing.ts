import { measureSpring, type SpringConfig, springConfig, springProgress } from "#runtime/spring.ts";

export type EasingFunction = (t: number) => number;

const NEWTON_STEPS = 8;
const BISECTION_STEPS = 40;
const PRECISION = 1e-7;
const SPRING_EASING_FPS = 60;

function linear(t: number): number {
  return t;
}

function quad(t: number): number {
  return t * t;
}

function cubic(t: number): number {
  return t * t * t;
}

function poly(power: number): EasingFunction {
  return (t) => t ** power;
}

function sin(t: number): number {
  return 1 - Math.cos((t * Math.PI) / 2);
}

function circle(t: number): number {
  return 1 - Math.sqrt(1 - t * t);
}

function exp(t: number): number {
  return 2 ** (10 * (t - 1));
}

function elastic(bounciness = 1): EasingFunction {
  const swing = bounciness * Math.PI;
  return (t) => 1 - Math.cos((t * Math.PI) / 2) ** 3 * Math.cos(t * swing);
}

function back(overshoot = 1.70158): EasingFunction {
  return (t) => t * t * ((overshoot + 1) * t - overshoot);
}

function bounce(t: number): number {
  if (t < 1 / 2.75) return 7.5625 * t * t;
  if (t < 2 / 2.75) {
    const u = t - 1.5 / 2.75;
    return 7.5625 * u * u + 0.75;
  }
  if (t < 2.5 / 2.75) {
    const u = t - 2.25 / 2.75;
    return 7.5625 * u * u + 0.9375;
  }
  const u = t - 2.625 / 2.75;
  return 7.5625 * u * u + 0.984375;
}

function curvePoint(at: number, first: number, second: number): number {
  const rest = 1 - at;
  return 3 * rest * rest * at * first + 3 * rest * at * at * second + at * at * at;
}

function curveSlope(at: number, first: number, second: number): number {
  const rest = 1 - at;
  return 3 * rest * rest * first + 6 * rest * at * (second - first) + 3 * at * at * (1 - second);
}

function curveAtX(x: number, x1: number, x2: number): number {
  let at = x;
  for (let i = 0; i < NEWTON_STEPS; i++) {
    const slope = curveSlope(at, x1, x2);
    if (Math.abs(slope) < PRECISION) break;
    const next = at - (curvePoint(at, x1, x2) - x) / slope;
    if (next < 0 || next > 1) break;
    at = next;
  }
  if (Math.abs(curvePoint(at, x1, x2) - x) < PRECISION) return at;
  let lo = 0;
  let hi = 1;
  at = x;
  for (let i = 0; i < BISECTION_STEPS; i++) {
    if (curvePoint(at, x1, x2) < x) lo = at;
    else hi = at;
    at = (lo + hi) / 2;
  }
  return at;
}

function bezier(x1: number, y1: number, x2: number, y2: number): EasingFunction {
  if (x1 === y1 && x2 === y2) return linear;
  return (t) => {
    if (t <= 0) return 0;
    if (t >= 1) return 1;
    return curvePoint(curveAtX(t, x1, x2), y1, y2);
  };
}

function easeIn(easing: EasingFunction): EasingFunction {
  return easing;
}

function easeOut(easing: EasingFunction): EasingFunction {
  return (t) => 1 - easing(1 - t);
}

function easeInOut(easing: EasingFunction): EasingFunction {
  return (t) => (t < 0.5 ? easing(t * 2) / 2 : 1 - easing((1 - t) * 2) / 2);
}

function springEasing(config: Partial<SpringConfig> = {}): EasingFunction {
  const resolved = springConfig(config);
  const seconds = measureSpring({ fps: SPRING_EASING_FPS, config: resolved }) / SPRING_EASING_FPS;
  return (t) => {
    if (t <= 0) return 0;
    if (t >= 1) return 1;
    return springProgress(resolved, t * seconds);
  };
}

export const Easing = {
  linear,
  ease: bezier(0.42, 0, 1, 1),
  quad,
  cubic,
  poly,
  sin,
  circle,
  exp,
  elastic,
  back,
  bounce,
  bezier,
  in: easeIn,
  out: easeOut,
  inOut: easeInOut,
  spring: springEasing,
} as const;
