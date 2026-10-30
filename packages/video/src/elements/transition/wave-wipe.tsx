import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "@remotion/transitions";
import type React from "react";
import { AbsoluteFill, Easing, interpolate } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { GrainGradientField } from "#video/parts/shader-fields.tsx";

type WaveWipeProps = ElementSettings["wave-wipe"];

function WaveWipePresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<WaveWipeProps>) {
  const { colors, colorBack, intensity, softness, noise, zoom, speed, direction } = passedProps;
  const sign = direction === "up" ? 1 : -1;
  const entering = presentationDirection === "entering";
  const p = presentationProgress;

  if (!entering) {
    const exitStyle: React.CSSProperties = {
      opacity: interpolate(p, [0.3, 0.5], [1, 0], {
        ...CLAMPED,
        easing: Easing.bezier(0.42, 0, 0.58, 1),
      }),
      translate: `0 ${interpolate(p, [0, 0.7], [0, -70 * sign], {
        ...CLAMPED,
        easing: Easing.in(Easing.cubic),
      })}%`,
    };
    return <AbsoluteFill style={exitStyle}>{children}</AbsoluteFill>;
  }

  const fieldOpacity = interpolate(p, [0.18, 0.45], [0, 1], {
    ...CLAMPED,
    easing: Easing.bezier(0.42, 0, 0.58, 1),
  });
  const drift = interpolate(p, [0, 1], [0, 0.7 * sign], {
    ...CLAMPED,
    easing: Easing.bezier(0.45, 0, 0.55, 1),
  });
  const rise = interpolate(p, [0.4, 0.82, 1], [100 * sign, -3.5 * sign, 0], {
    ...CLAMPED,
    easing: Easing.out(Easing.cubic),
  });

  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ opacity: fieldOpacity, pointerEvents: "none" }}>
        <GrainGradientField
          speed={speed}
          shape="wave"
          colors={colors}
          colorBack={colorBack}
          intensity={intensity}
          softness={softness}
          noise={noise}
          scale={zoom}
          offsetY={drift}
        />
      </AbsoluteFill>
      <AbsoluteFill style={{ translate: `0 ${rise}%` }}>{children}</AbsoluteFill>
    </AbsoluteFill>
  );
}

export function waveWipe(props: WaveWipeProps): TransitionPresentation<WaveWipeProps> {
  return {
    component: WaveWipePresentation,
    props,
  };
}
