import type React from "react";
import { AbsoluteFill, Easing, interpolate } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";

type PushThroughProps = ElementSettings["push-through"];

function PushThroughPresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<PushThroughProps>) {
  const { zoom, blur, approachFrom } = passedProps;
  const entering = presentationDirection === "entering";
  const p = presentationProgress;

  if (!entering) {
    const exitStyle: React.CSSProperties = {
      opacity: interpolate(p, [0.4, 0.62], [1, 0], {
        ...CLAMPED,
        easing: Easing.bezier(0.42, 0, 0.58, 1),
      }),
      scale: `${interpolate(p, [0, 0.65], [1, zoom], {
        ...CLAMPED,
        easing: Easing.in(Easing.cubic),
      })}`,
      filter: `blur(${interpolate(p, [0.15, 0.6], [0, blur], CLAMPED)}px)`,
    };
    return <AbsoluteFill style={exitStyle}>{children}</AbsoluteFill>;
  }

  const childStyle: React.CSSProperties = {
    opacity: interpolate(p, [0.3, 0.5], [0, 1], CLAMPED),
    scale: `${interpolate(p, [0.3, 0.88, 1], [approachFrom, 1.02, 1], {
      ...CLAMPED,
      easing: Easing.out(Easing.cubic),
    })}`,
    filter: `blur(${interpolate(p, [0.3, 0.75], [blur * 0.7, 0], CLAMPED)}px)`,
  };

  return <AbsoluteFill style={childStyle}>{children}</AbsoluteFill>;
}

export function pushThrough(props: PushThroughProps): TransitionPresentation<PushThroughProps> {
  return {
    component: PushThroughPresentation,
    props,
  };
}
