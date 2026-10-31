import { useFilm } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useId, useMemo } from "react";
import {
  DISPLACEMENT_DEFAULTS,
  DISPLACEMENT_EPSILON,
  displacementFrame,
  maskFrame,
  MASK_DEFAULTS,
  prepareMaskMorph,
  type DisplacementCoreConfig,
  type DisplacementMaskConfig,
} from "#video/elements/transition/transition-morph-shape/displacement-transition.ts";
import { morphPath } from "#video/elements/transition/transition-morph-shape/path-morph.ts";
import type { TransitionPresentation } from "#video/lib/transitions/presentation.ts";
import { transitionPhase } from "#video/lib/transitions/transition-timing.ts";

export type DisplacementPresentationProps = DisplacementCoreConfig & {
  mask?: DisplacementMaskConfig;
  fade?: boolean;
  lead?: number;
  affect?: "both" | "entering";
};

function DisplacementPresentationComponent({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: React.ComponentProps<
  NonNullable<TransitionPresentation<DisplacementPresentationProps>["component"]>
>) {
  const { width, height } = useFilm();
  const rawId = useId();
  const domId = rawId.replace(/:/g, "");

  const mask = passedProps.mask;
  const hasMask = mask !== undefined;
  const fade = passedProps.fade ?? !hasMask;
  const lead = passedProps.lead ?? 1;
  const affect = passedProps.affect ?? (hasMask ? "entering" : "both");

  const phase = transitionPhase(presentationProgress, presentationDirection, {
    lead,
    fade,
  });

  const opacity = fade && phase.isEntering ? phase.opacity : 1;

  const state = displacementFrame(phase.displace, Math.min(width, height), passedProps);
  const maskState = maskFrame(phase.displace, { width, height }, mask ?? {});

  const maskFrom = mask?.from;
  const maskTo = mask?.to;
  const maskPair = useMemo(
    () => (hasMask ? prepareMaskMorph({ from: maskFrom, to: maskTo }) : null),
    [hasMask, maskFrom, maskTo],
  );

  const wantsFilter =
    state.scale >= DISPLACEMENT_EPSILON && (affect === "both" || phase.isEntering);
  const wantsMask = hasMask && phase.isEntering && !maskState.isComplete;

  const frequency = passedProps.frequency ?? DISPLACEMENT_DEFAULTS.frequency;
  const octaves = passedProps.octaves ?? DISPLACEMENT_DEFAULTS.octaves;
  const turbulenceType = passedProps.turbulenceType ?? DISPLACEMENT_DEFAULTS.turbulenceType;

  if (hasMask && !phase.isEntering) {
    return <AbsoluteFill>{children}</AbsoluteFill>;
  }

  if (!wantsFilter && !wantsMask) {
    return <AbsoluteFill style={{ opacity }}>{children}</AbsoluteFill>;
  }

  const maskD = maskPair && wantsMask ? morphPath(maskState.coverage, maskPair) : null;

  return (
    <AbsoluteFill
      style={{
        opacity,
        ...(wantsMask ? { clipPath: `url(#${domId}-clip)` } : {}),
      }}
    >
      <svg width={0} height={0} style={{ position: "absolute" }}>
        <defs>
          {wantsFilter ? (
            <filter id={`${domId}-warp`} x="-30%" y="-30%" width="160%" height="160%">
              <feTurbulence
                type={turbulenceType}
                baseFrequency={frequency}
                numOctaves={octaves}
                seed={state.seed}
                result="noise"
              />
              <feDisplacementMap
                in="SourceGraphic"
                in2="noise"
                scale={state.scale}
                xChannelSelector="R"
                yChannelSelector="G"
              />
              {state.blur > 0.1 ? <feGaussianBlur stdDeviation={state.blur} /> : null}
            </filter>
          ) : null}
          {maskD ? (
            <clipPath id={`${domId}-clip`} clipPathUnits="userSpaceOnUse">
              <path
                d={maskD}
                transform={`translate(${maskState.centerX} ${maskState.centerY}) scale(${maskState.size}) translate(-0.5 -0.5)`}
              />
            </clipPath>
          ) : null}
        </defs>
      </svg>
      <AbsoluteFill
        style={
          wantsFilter
            ? {
                filter: `url(#${domId}-warp)`,
                scale: `${state.overscan}`,
              }
            : undefined
        }
      >
        {children}
      </AbsoluteFill>
    </AbsoluteFill>
  );
}

export function displacementPresentation(
  props: DisplacementPresentationProps = {},
): TransitionPresentation<DisplacementPresentationProps> {
  return { component: DisplacementPresentationComponent, props };
}

export { MASK_DEFAULTS };
