import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { spring, type SpringConfig } from "@opsis/runtime/spring.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { springBouncy, springSmooth, springSnappy } from "#video/lib/motion/springs.ts";
import { EASING_ENTER, EASING_EXIT } from "#video/lib/motion/timing.ts";

type SpringPreset = "smooth" | "snappy" | "bouncy";

export type MotionSpring = boolean | SpringPreset | Partial<SpringConfig>;

const SPRING_PRESETS: Record<SpringPreset, SpringConfig> = {
  smooth: springSmooth,
  snappy: springSnappy,
  bouncy: springBouncy,
};

export function resolveSpringConfig(value: MotionSpring): SpringConfig {
  if (value === true || value === false) return springSnappy;
  if (typeof value === "string") return SPRING_PRESETS[value] ?? springSnappy;
  return { ...springSnappy, ...value };
}

const EASING_ARRIVE = Easing.bezier(0.33, 1, 0.68, 1);

const ENTER_OPACITY_LEAD = 0.55;
const EXIT_OPACITY_LEAD = 0.7;
const EXIT_DURATION_RATIO = 0.7;
const MIN_EXIT_FRAMES = 6;
const DEFAULT_EXIT_TRAVEL = 0.6;

const DEFAULT_ENTER_FRAMES = 30;

export function defaultExitFrames(durationInFrames: number): number {
  return Math.max(MIN_EXIT_FRAMES, Math.round(durationInFrames * EXIT_DURATION_RATIO));
}

export type EnterExitOptions = {
  durationInFrames?: number;
  delayInFrames?: number;
  spring?: MotionSpring;
  exit?: boolean;
  exitInFrames?: number;
  exitAtInFrames?: number;
  exitTravel?: number;
  exitDirection?: "reverse" | "continue";
};

export type MotionState = {
  motion: number;
  displace: number;
  opacity: number;
  sign: 1 | -1;
  exiting: boolean;
  exitProgress: number;
  enterTime: number;
};

export function useEnterExit({
  durationInFrames = DEFAULT_ENTER_FRAMES,
  delayInFrames = 0,
  spring: springProp,
  exit = false,
  exitInFrames,
  exitAtInFrames,
  exitTravel = DEFAULT_EXIT_TRAVEL,
  exitDirection = "reverse",
}: EnterExitOptions): MotionState {
  const frame = useFrame();
  const { fps, durationInFrames: windowFrames } = useFilm();

  const duration = Math.max(1, Math.round(durationInFrames));
  const delay = Math.max(0, Math.round(delayInFrames));

  const enterTime = interpolate(frame, [delay, delay + duration], [0, 1], CLAMPED);

  const enterMotion = springProp
    ? spring({
        frame,
        fps,
        config: resolveSpringConfig(springProp),
        delay,
        durationInFrames: duration,
      })
    : interpolate(enterTime, [0, 1], [0, 1], {
        easing: EASING_ARRIVE,
        ...CLAMPED,
      });

  const enterOpacity = interpolate(enterTime, [0, ENTER_OPACITY_LEAD], [0, 1], {
    easing: EASING_ENTER,
    ...CLAMPED,
  });

  const exitFrames =
    exitInFrames === undefined
      ? defaultExitFrames(duration)
      : Math.max(MIN_EXIT_FRAMES, Math.round(exitInFrames));
  const hasWindow = Number.isFinite(windowFrames);
  const requested = exitAtInFrames ?? (hasWindow ? windowFrames - exitFrames : null);
  const wantsExit = exit || exitInFrames !== undefined || exitAtInFrames !== undefined;

  if (!wantsExit || requested === null) {
    return {
      motion: enterMotion,
      displace: 1 - enterMotion,
      opacity: enterOpacity,
      sign: 1,
      exiting: false,
      exitProgress: 0,
      enterTime,
    };
  }

  const exitStart = Math.max(requested, delay + duration);
  const exitTime = interpolate(frame, [exitStart, exitStart + exitFrames], [0, 1], CLAMPED);
  const exitProgress = interpolate(exitTime, [0, 1], [0, 1], {
    easing: EASING_EXIT,
    ...CLAMPED,
  });
  const exitOpacity = interpolate(exitTime, [0, EXIT_OPACITY_LEAD], [1, 0], {
    easing: EASING_EXIT,
    ...CLAMPED,
  });
  const exiting = frame >= exitStart;

  return {
    motion: exiting ? 1 - exitProgress : enterMotion,
    displace: exiting ? exitProgress * exitTravel : 1 - enterMotion,
    opacity: enterOpacity * exitOpacity,
    sign: exiting && exitDirection === "continue" ? -1 : 1,
    exiting,
    exitProgress,
    enterTime,
  };
}
