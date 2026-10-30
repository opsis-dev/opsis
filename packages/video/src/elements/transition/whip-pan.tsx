import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "@remotion/transitions";
import type React from "react";
import { AbsoluteFill, Easing, interpolate } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

type WhipPanProps = ElementSettings["whip-pan"];

function WhipPanPresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<WhipPanProps>) {
  const { direction, blur, stretch: stretchPercent } = passedProps;
  const entering = presentationDirection === "entering";
  const p = presentationProgress;

  const travel = interpolate(p, [0, 1], [0, 1], {
    ...CLAMPED,
    easing: Easing.bezier(0.7, 0, 0.2, 1),
  });
  const velocity = Math.sin(Math.PI * travel);

  const axis: "x" | "y" = direction === "left" || direction === "right" ? "x" : "y";
  const sign = direction === "left" || direction === "up" ? -1 : 1;

  const offset = entering ? (travel - 1) * 110 * sign : travel * 110 * sign;
  const stretch = 1 + (velocity * stretchPercent) / 100;

  const translate = axis === "x" ? `${offset}%` : `0 ${offset}%`;
  const smear = axis === "x" ? `${stretch} 1` : `1 ${stretch}`;

  const style: React.CSSProperties = {
    translate,
    scale: smear,
    filter: `blur(${velocity * blur}px)`,
  };

  return <AbsoluteFill style={style}>{children}</AbsoluteFill>;
}

export function whipPan(props: WhipPanProps): TransitionPresentation<WhipPanProps> {
  return {
    component: WhipPanPresentation,
    props,
  };
}
