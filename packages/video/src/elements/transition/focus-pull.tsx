import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "@remotion/transitions";
import type React from "react";
import { AbsoluteFill, Easing, interpolate } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";

const clampOpts = {
  extrapolateLeft: "clamp" as const,
  extrapolateRight: "clamp" as const,
};

type FocusPullProps = ElementSettings["focus-pull"];

const ENTER_GLOW_SHARE = 25 / 30;
const ENTER_ZOOM_SHARE = 0.6;

function FocusPullPresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<FocusPullProps>) {
  const { blur, glow, zoom } = passedProps;
  const outGlow = 1 + glow / 100;
  const inGlow = 1 + (glow / 100) * ENTER_GLOW_SHARE;
  const entering = presentationDirection === "entering";
  const p = presentationProgress;

  if (!entering) {
    const exitStyle: React.CSSProperties = {
      opacity: interpolate(p, [0.42, 0.68], [1, 0], {
        ...clampOpts,
        easing: Easing.bezier(0.42, 0, 0.58, 1),
      }),
      scale: `${interpolate(p, [0, 0.6], [1, 1 + zoom / 100], {
        ...clampOpts,
        easing: Easing.bezier(0.42, 0, 0.58, 1),
      })}`,
      filter: `blur(${interpolate(p, [0.05, 0.55], [0, blur], {
        ...clampOpts,
        easing: Easing.in(Easing.quad),
      })}px) brightness(${interpolate(p, [0.1, 0.55], [1, outGlow], clampOpts)})`,
    };
    return <AbsoluteFill style={exitStyle}>{children}</AbsoluteFill>;
  }

  const childStyle: React.CSSProperties = {
    opacity: interpolate(p, [0.32, 0.52], [0, 1], clampOpts),
    scale: `${interpolate(p, [0.35, 1], [1 - (zoom / 100) * ENTER_ZOOM_SHARE, 1], {
      ...clampOpts,
      easing: Easing.out(Easing.cubic),
    })}`,
    filter: `blur(${interpolate(p, [0.35, 0.9], [blur, 0], {
      ...clampOpts,
      easing: Easing.out(Easing.quad),
    })}px) brightness(${interpolate(p, [0.4, 0.85], [inGlow, 1], clampOpts)})`,
  };

  return <AbsoluteFill style={childStyle}>{children}</AbsoluteFill>;
}

export function focusPull(props: FocusPullProps): TransitionPresentation<FocusPullProps> {
  return {
    component: FocusPullPresentation,
    props,
  };
}
