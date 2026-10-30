import { Easing, interpolate } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { PresentationDirection } from "#video/lib/transitions/presentation.ts";

export type TransitionVariant = "linear" | "editorial";

export type TransitionTiming = {
  durationInFrames: number;
  progressAt: (frame: number) => number;
};

const EASING_CUT = Easing.bezier(0.45, 0, 0.55, 1);

export function resolveTransitionTiming({
  durationInFrames,
  variant = "editorial",
}: {
  durationInFrames: number;
  variant?: TransitionVariant;
}): TransitionTiming {
  const easing = variant === "editorial" ? EASING_CUT : undefined;
  return {
    durationInFrames,
    progressAt: (frame) =>
      interpolate(frame, [0, durationInFrames], [0, 1], { ...CLAMPED, easing }),
  };
}

export type TransitionDirection = PresentationDirection;

export type TransitionPhase = {
  isEntering: boolean;
  displace: number;
  opacity: number;
  progress: number;
};

export type TransitionPhaseOptions = {
  lead?: number;
  fade?: boolean;
};

export function transitionPhase(
  presentationProgress: number,
  presentationDirection: TransitionDirection,
  { lead = 0.72, fade = false }: TransitionPhaseOptions = {},
): TransitionPhase {
  const isEntering = presentationDirection === "entering";

  const spansWindow = lead >= 1;

  if (isEntering) {
    const arrived = spansWindow
      ? presentationProgress
      : interpolate(presentationProgress, [0, lead, 1], [0, 1, 1], CLAMPED);

    return {
      isEntering,
      displace: 1 - arrived,
      opacity: fade ? interpolate(presentationProgress, [0, 0.42, 1], [0, 0.72, 1], CLAMPED) : 1,
      progress: presentationProgress,
    };
  }

  const left = spansWindow
    ? presentationProgress
    : interpolate(presentationProgress, [0, 1 - lead, 1], [0, 0, 1], CLAMPED);

  return {
    isEntering,
    displace: left,
    opacity: fade ? interpolate(presentationProgress, [0, 0.58, 1], [1, 0.42, 0], CLAMPED) : 1,
    progress: presentationProgress,
  };
}
