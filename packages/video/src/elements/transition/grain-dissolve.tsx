import type React from "react";
import { AbsoluteFill, Easing, interpolate } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";
import { GrainGradientField } from "#video/parts/shader-fields.tsx";

type GrainDissolveProps = ElementSettings["grain-dissolve"];

function GrainDissolvePresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<GrainDissolveProps>) {
  const { colors, colorBack, shape, noise, zoom, speed, sceneBlur, rotation } = passedProps;
  const entering = presentationDirection === "entering";
  const p = presentationProgress;

  if (!entering) {
    const exitStyle: React.CSSProperties = {
      opacity: interpolate(p, [0.14, 0.34], [1, 0], CLAMPED),
      filter: `blur(${interpolate(p, [0.06, 0.34], [0, sceneBlur], CLAMPED)}px)`,
    };
    return <AbsoluteFill style={exitStyle}>{children}</AbsoluteFill>;
  }

  const fieldOpacity = interpolate(p, [0, 0.26], [0, 1], {
    ...CLAMPED,
    easing: Easing.bezier(0.42, 0, 0.58, 1),
  });
  const intensity = interpolate(p, [0.08, 0.92], [0, 1], {
    ...CLAMPED,
    easing: Easing.bezier(0.42, 0, 0.58, 1),
  });
  const softness = interpolate(p, [0.08, 0.92], [0, 1], {
    ...CLAMPED,
    easing: Easing.bezier(0.42, 0, 0.58, 1),
  });
  const fieldScale = zoom * interpolate(p, [0, 1], [0.9, 1.15], CLAMPED);

  const childStyle: React.CSSProperties = {
    opacity: interpolate(p, [0.7, 0.86], [0, 1], CLAMPED),
    scale: `${interpolate(p, [0.7, 1], [1.05, 1], {
      ...CLAMPED,
      easing: Easing.out(Easing.cubic),
    })}`,
    filter: `blur(${interpolate(p, [0.7, 0.92], [sceneBlur, 0], CLAMPED)}px)`,
  };

  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ opacity: fieldOpacity, pointerEvents: "none" }}>
        <GrainGradientField
          speed={speed}
          colors={colors}
          colorBack={colorBack}
          shape={shape}
          noise={noise}
          intensity={intensity}
          softness={softness}
          scale={fieldScale}
          rotation={rotation}
        />
      </AbsoluteFill>
      <AbsoluteFill style={childStyle}>{children}</AbsoluteFill>
    </AbsoluteFill>
  );
}

export function grainDissolve(
  props: GrainDissolveProps,
): TransitionPresentation<GrainDissolveProps> {
  return {
    component: GrainDissolvePresentation,
    props,
  };
}
