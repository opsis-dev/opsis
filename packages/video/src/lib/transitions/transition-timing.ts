import { linearTiming, springTiming } from "@remotion/transitions";
import type { TransitionPresentationComponentProps } from "@remotion/transitions";
import { Easing, interpolate } from "remotion";
import { springSmooth } from "#video/lib/motion/springs.ts";

export type TransitionVariant = "linear" | "spring" | "editorial";

const EASING_CUT = Easing.bezier(0.45, 0, 0.55, 1);

export function resolveTransitionTiming({
  durationInFrames,
  variant = "editorial",
}: {
  durationInFrames: number;
  variant?: TransitionVariant;
}) {
  if (variant === "spring") {
    return springTiming({ config: springSmooth, durationInFrames });
  }

  return linearTiming({
    durationInFrames,
    easing: variant === "editorial" ? EASING_CUT : undefined,
  });
}

export type TransitionDirection = TransitionPresentationComponentProps<
  Record<string, unknown>
>["presentationDirection"];

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
      : interpolate(presentationProgress, [0, lead, 1], [0, 1, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });

    return {
      isEntering,
      displace: 1 - arrived,
      opacity: fade
        ? interpolate(presentationProgress, [0, 0.42, 1], [0, 0.72, 1], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
          })
        : 1,
      progress: presentationProgress,
    };
  }

  const left = spansWindow
    ? presentationProgress
    : interpolate(presentationProgress, [0, 1 - lead, 1], [0, 0, 1], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
      });

  return {
    isEntering,
    displace: left,
    opacity: fade
      ? interpolate(presentationProgress, [0, 0.58, 1], [1, 0.42, 0], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        })
      : 1,
    progress: presentationProgress,
  };
}
