import type { TransitionPresentation } from "@remotion/transitions";
import { useId, useMemo } from "react";
import { AbsoluteFill, interpolate } from "remotion";
import {
  resolveTransitionTiming,
  transitionPhase,
  type TransitionVariant,
} from "#video/lib/transitions/transition-timing.ts";

export type ChromaticAberrationAxis = "horizontal" | "vertical";

export type ChromaticAberrationWipeProps = {
  intensity?: number;
  axis?: ChromaticAberrationAxis;
  slide?: number;
};

const CHANNEL_MATRIX = {
  red: "1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0",
  green: "0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0",
  blue: "0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0",
} as const;

const CHANNEL_OFFSET: Record<keyof typeof CHANNEL_MATRIX, number> = {
  red: 1,
  green: 0,
  blue: -1,
};

const ChromaticPresentation: React.FC<
  React.ComponentProps<
    NonNullable<TransitionPresentation<ChromaticAberrationWipeProps>["component"]>
  >
> = ({
  children,
  presentationProgress,
  presentationDirection,
  passedProps: { intensity = 12, axis = "horizontal", slide = 1 },
}) => {
  const filterId = useId().replace(/:/g, "");
  const phase = transitionPhase(presentationProgress, presentationDirection, {
    lead: 1,
  });

  const separation = interpolate(phase.displace, [0, 0.5, 1], [0, intensity, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const travel = phase.displace * (phase.isEntering ? 100 : -100) * slide;

  const shift = useMemo(
    () => (axis === "horizontal" ? `${travel}% 0%` : `0% ${travel}%`),
    [axis, travel],
  );

  const channels = useMemo(
    () =>
      (Object.keys(CHANNEL_MATRIX) as (keyof typeof CHANNEL_MATRIX)[]).map((channel) => {
        const offset = CHANNEL_OFFSET[channel] * separation;

        return {
          channel,
          style: {
            filter: `url(#${filterId}-${channel})`,
            mixBlendMode: "screen" as const,
            translate: axis === "horizontal" ? `${offset}px 0px` : `0px ${offset}px`,
          },
        };
      }),
    [axis, filterId, separation],
  );

  const split = separation >= 0.1;
  const shown = split ? channels : [{ channel: channels[0].channel, style: {} }];

  return (
    <AbsoluteFill
      style={{
        opacity: phase.opacity,
        translate: shift,
        isolation: "isolate",
      }}
    >
      <svg width={0} height={0} style={{ position: "absolute" }}>
        <defs>
          {(Object.keys(CHANNEL_MATRIX) as (keyof typeof CHANNEL_MATRIX)[]).map((channel) => (
            <filter key={channel} id={`${filterId}-${channel}`}>
              <feColorMatrix type="matrix" values={CHANNEL_MATRIX[channel]} />
            </filter>
          ))}
        </defs>
      </svg>
      {shown.map(({ channel, style }) => (
        <AbsoluteFill key={channel} style={style}>
          {children}
        </AbsoluteFill>
      ))}
    </AbsoluteFill>
  );
};

export function chromaticAberrationWipe(
  props: ChromaticAberrationWipeProps = {},
): TransitionPresentation<ChromaticAberrationWipeProps> {
  return { component: ChromaticPresentation, props };
}

export type TransitionChromaticAberrationWipeConfig = {
  durationInFrames?: number;
  intensity?: number;
  axis?: ChromaticAberrationAxis;
  slide?: number;
  variant?: TransitionVariant;
};

export function transitionChromaticAberrationWipe({
  durationInFrames = 14,
  intensity = 12,
  axis = "horizontal",
  slide = 1,
  variant = "editorial",
}: TransitionChromaticAberrationWipeConfig = {}) {
  return {
    presentation: chromaticAberrationWipe({ intensity, axis, slide }),
    timing: resolveTransitionTiming({ durationInFrames, variant }),
  };
}

export function getTransitionChromaticAberrationWipeDuration(
  config: TransitionChromaticAberrationWipeConfig = {},
  fps: number,
): number {
  return transitionChromaticAberrationWipe(config).timing.getDurationInFrames({
    fps,
  });
}
