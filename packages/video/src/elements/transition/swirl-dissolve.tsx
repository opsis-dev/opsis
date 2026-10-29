import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "@remotion/transitions";
import type React from "react";
import { AbsoluteFill, Easing, interpolate } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import { SwirlField } from "#video/parts/shader-fields.tsx";

const clampOpts = {
  extrapolateLeft: "clamp" as const,
  extrapolateRight: "clamp" as const,
};

type SwirlDissolveProps = ElementSettings["swirl-dissolve"];

function SwirlDissolvePresentation({
  children,
  presentationProgress,
  presentationDirection,
  presentationDurationInFrames,
  passedProps,
}: TransitionPresentationComponentProps<SwirlDissolveProps>) {
  const { colors, colorBack, bandCount, softness, speed, noise } = passedProps;
  const entering = presentationDirection === "entering";
  const p =
    presentationDurationInFrames > 1
      ? Math.min(
          1,
          (presentationProgress * presentationDurationInFrames) /
            (presentationDurationInFrames - 1),
        )
      : 1;

  const twist = interpolate(p, [0.1, 0.42, 0.64, 1], [1, 0, 0, 1], {
    ...clampOpts,
    easing: Easing.bezier(0.42, 0, 0.58, 1),
  });

  const childStyle: React.CSSProperties = entering
    ? {
        opacity: interpolate(p, [0.77, 0.87], [0, 1], clampOpts),
        scale: `${interpolate(p, [0.77, 1], [0.3, 1], {
          ...clampOpts,
          easing: Easing.out(Easing.cubic),
        })}`,
        filter: `blur(${interpolate(p, [0.77, 0.93], [10, 0], clampOpts)}px)`,
      }
    : { opacity: interpolate(p, [0.06, 0.13], [1, 0], clampOpts) };

  return (
    <AbsoluteFill>
      {entering ? (
        <AbsoluteFill style={{ pointerEvents: "none" }}>
          <SwirlField
            speed={speed}
            twist={twist}
            colors={colors}
            colorBack={colorBack}
            bandCount={bandCount}
            softness={softness}
            noise={noise}
          />
        </AbsoluteFill>
      ) : null}
      <AbsoluteFill style={childStyle}>{children}</AbsoluteFill>
    </AbsoluteFill>
  );
}

export function swirlDissolve(
  props: SwirlDissolveProps,
): TransitionPresentation<SwirlDissolveProps> {
  return {
    component: SwirlDissolvePresentation,
    props,
  };
}
