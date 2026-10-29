import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "@remotion/transitions";
import type React from "react";
import { AbsoluteFill, Easing, interpolate } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import { GrainGradientField } from "#video/parts/shader-fields.tsx";

const clampOpts = {
  extrapolateLeft: "clamp" as const,
  extrapolateRight: "clamp" as const,
};

type RippleZoomProps = ElementSettings["ripple-zoom"];

function RippleZoomPresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<RippleZoomProps>) {
  const { colors, colorBack, intensity, softness, noise, zoom, speed, originX } = passedProps;
  const entering = presentationDirection === "entering";
  const p = presentationProgress;

  if (!entering) {
    const exitStyle: React.CSSProperties = {
      opacity: interpolate(p, [0.35, 0.55], [1, 0], {
        ...clampOpts,
        easing: Easing.bezier(0.42, 0, 0.58, 1),
      }),
      scale: `${interpolate(p, [0.3, 0.62], [1, 1.6], {
        ...clampOpts,
        easing: Easing.in(Easing.cubic),
      })}`,
    };
    return <AbsoluteFill style={exitStyle}>{children}</AbsoluteFill>;
  }

  const fieldOpacity = interpolate(p, [0.08, 0.32], [0, 1], {
    ...clampOpts,
    easing: Easing.bezier(0.42, 0, 0.58, 1),
  });
  const fieldScale = interpolate(p, [0.32, 1], [0.2, zoom], {
    ...clampOpts,
    easing: Easing.bezier(0.7, 0, 0.3, 1),
  });

  const childStyle: React.CSSProperties = {
    opacity: interpolate(p, [0.45, 0.62], [0, 1], clampOpts),
    scale: `${interpolate(p, [0.42, 0.97], [0.2, 1], {
      ...clampOpts,
      easing: Easing.bezier(0.33, 1, 0.68, 1),
    })}`,
    filter: `blur(${interpolate(p, [0.42, 0.8], [8, 0], clampOpts)}px)`,
  };

  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ opacity: fieldOpacity, pointerEvents: "none" }}>
        <GrainGradientField
          speed={speed}
          shape="ripple"
          colors={colors}
          colorBack={colorBack}
          intensity={intensity}
          softness={softness}
          noise={noise}
          scale={fieldScale}
          offsetX={(originX - 0.5) * 2}
        />
      </AbsoluteFill>
      <AbsoluteFill style={childStyle}>{children}</AbsoluteFill>
    </AbsoluteFill>
  );
}

export function rippleZoom(props: RippleZoomProps): TransitionPresentation<RippleZoomProps> {
  return {
    component: RippleZoomPresentation,
    props,
  };
}
