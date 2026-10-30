import {
  displacementPresentation,
  type DisplacementPresentationProps,
} from "#video/elements/transition/transition-morph-shape/displacement-presentation.tsx";
import type { ElementSettings, MorphShape } from "#video/generated/spec.ts";
import type { TransitionPresentation } from "#video/lib/transitions/presentation.ts";

type MorphShapeProps = ElementSettings["transition-morph-shape"];

const SHAPE_OVERSHOOT: Record<MorphShape, number> = {
  circle: 1.06,
  squircle: 1.1,
  square: 1.06,
  blob: 1.28,
  diamond: 1.42,
  triangle: 2.1,
};

function morphProps({
  from,
  to,
  originX,
  originY,
  overshoot,
  warp,
  warpDetail,
  seed,
}: MorphShapeProps): DisplacementPresentationProps {
  return {
    scale: warp,
    frequency: warpDetail,
    seed,
    mask: {
      from,
      to,
      originX,
      originY,
      overshoot: overshoot ?? SHAPE_OVERSHOOT[to],
    },
    fade: false,
    lead: 1,
    affect: "entering",
  };
}

export function morphShape(
  props: MorphShapeProps,
): TransitionPresentation<DisplacementPresentationProps> {
  return displacementPresentation(morphProps(props));
}
