import type { TransitionPresentation } from "@remotion/transitions";
import { AbsoluteFill, interpolate } from "remotion";
import {
  resolveTransitionTiming,
  transitionPhase,
  type TransitionVariant,
} from "#video/lib/transitions/transition-timing.ts";

export type FadeProps = {
  dipTo?: string;
};

function FadePresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps: { dipTo },
}: React.ComponentProps<NonNullable<TransitionPresentation<FadeProps>["component"]>>) {
  const phase = transitionPhase(presentationProgress, presentationDirection, {
    lead: 1,
    fade: true,
  });

  if (dipTo === undefined) {
    return <AbsoluteFill style={{ opacity: phase.opacity }}>{children}</AbsoluteFill>;
  }
  const veil = interpolate(phase.displace, [0, 0.5, 1], [0, 1, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill>
      {children}
      <AbsoluteFill style={{ background: dipTo, opacity: veil }} />
    </AbsoluteFill>
  );
}

export function crossfade(props: FadeProps = {}): TransitionPresentation<FadeProps> {
  return { component: FadePresentation, props };
}

export type TransitionFadeConfig = {
  durationInFrames?: number;
  variant?: TransitionVariant;
  dipTo?: string;
};

export function transitionFade({
  durationInFrames = 18,
  variant = "editorial",
  dipTo,
}: TransitionFadeConfig = {}) {
  return {
    presentation: crossfade({ dipTo }),
    timing: resolveTransitionTiming({ durationInFrames, variant }),
  };
}

export function getTransitionFadeDuration(config: TransitionFadeConfig = {}, fps: number): number {
  return transitionFade(config).timing.getDurationInFrames({ fps });
}
