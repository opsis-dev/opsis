import type { TransitionPresentation } from "@remotion/transitions";
import {
  displacementPresentation,
  type DisplacementPresentationProps,
} from "#video/elements/transition/transition-morph-shape/displacement-presentation.tsx";
import type { MorphShapeName } from "#video/elements/transition/transition-morph-shape/path-morph.ts";
import {
  resolveTransitionTiming,
  type TransitionVariant,
} from "#video/lib/transitions/transition-timing.ts";

export type MorphShapeProps = {
  from?: MorphShapeName | string;
  to?: MorphShapeName | string;
  originX?: number;
  originY?: number;
  overshoot?: number;
  warp?: number;
};

export const SHAPE_OVERSHOOT: Record<string, number> = {
  circle: 1.06,
  squircle: 1.1,
  square: 1.06,
  blob: 1.28,
  diamond: 1.42,
  triangle: 2.1,
};

function resolveOvershoot(to: MorphShapeName | string): number {
  return SHAPE_OVERSHOOT[to] ?? SHAPE_OVERSHOOT.circle;
}

export const MORPH_SHAPE_DEFAULTS = {
  from: "circle",
  to: "diamond",
  originX: 0.5,
  originY: 0.5,
  warp: 0,
} as const satisfies Omit<Required<MorphShapeProps>, "overshoot">;

function morphProps(props: MorphShapeProps): DisplacementPresentationProps {
  const to = props.to ?? MORPH_SHAPE_DEFAULTS.to;
  return {
    scale: props.warp ?? MORPH_SHAPE_DEFAULTS.warp,
    mask: {
      from: props.from ?? MORPH_SHAPE_DEFAULTS.from,
      to,
      originX: props.originX ?? MORPH_SHAPE_DEFAULTS.originX,
      originY: props.originY ?? MORPH_SHAPE_DEFAULTS.originY,
      overshoot: props.overshoot ?? resolveOvershoot(to),
    },
    fade: false,
    lead: 1,
    affect: "entering",
  };
}

export function morphShape(
  props: MorphShapeProps = {},
): TransitionPresentation<DisplacementPresentationProps> {
  return displacementPresentation(morphProps(props));
}

export type TransitionMorphShapeConfig = MorphShapeProps & {
  durationInFrames?: number;
  variant?: TransitionVariant;
};

export function transitionMorphShape({
  durationInFrames = 24,
  variant = "editorial",
  ...presentationProps
}: TransitionMorphShapeConfig = {}) {
  return {
    presentation: morphShape(presentationProps),
    timing: resolveTransitionTiming({ durationInFrames, variant }),
  };
}

export function getTransitionMorphShapeDuration(
  config: TransitionMorphShapeConfig = {},
  fps: number,
): number {
  return transitionMorphShape(config).timing.getDurationInFrames({ fps });
}
