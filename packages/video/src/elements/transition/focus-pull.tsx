import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type React from "react";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";

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
        ...CLAMPED,
        easing: Easing.bezier(0.42, 0, 0.58, 1),
      }),
      scale: `${interpolate(p, [0, 0.6], [1, 1 + zoom / 100], {
        ...CLAMPED,
        easing: Easing.bezier(0.42, 0, 0.58, 1),
      })}`,
      filter: `blur(${interpolate(p, [0.05, 0.55], [0, blur], {
        ...CLAMPED,
        easing: Easing.in(Easing.quad),
      })}px) brightness(${interpolate(p, [0.1, 0.55], [1, outGlow], CLAMPED)})`,
    };
    return <AbsoluteFill style={exitStyle}>{children}</AbsoluteFill>;
  }

  const childStyle: React.CSSProperties = {
    opacity: interpolate(p, [0.32, 0.52], [0, 1], CLAMPED),
    scale: `${interpolate(p, [0.35, 1], [1 - (zoom / 100) * ENTER_ZOOM_SHARE, 1], {
      ...CLAMPED,
      easing: Easing.out(Easing.cubic),
    })}`,
    filter: `blur(${interpolate(p, [0.35, 0.9], [blur, 0], {
      ...CLAMPED,
      easing: Easing.out(Easing.quad),
    })}px) brightness(${interpolate(p, [0.4, 0.85], [inGlow, 1], CLAMPED)})`,
  };

  return <AbsoluteFill style={childStyle}>{children}</AbsoluteFill>;
}

export function focusPull(props: FocusPullProps): TransitionPresentation<FocusPullProps> {
  return {
    component: FocusPullPresentation,
    props,
  };
}
