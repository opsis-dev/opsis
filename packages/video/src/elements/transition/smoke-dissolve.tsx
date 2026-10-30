import type React from "react";
import { AbsoluteFill, Easing, interpolate } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";
import { SmokeRingField } from "#video/parts/shader-fields.tsx";

type SmokeDissolveProps = ElementSettings["smoke-dissolve"];

function SmokeDissolvePresentation({
  children,
  presentationProgress,
  presentationDirection,
  presentationDurationInFrames,
  passedProps,
}: TransitionPresentationComponentProps<SmokeDissolveProps>) {
  const { colorBack, colors, speed, detail, originX } = passedProps;
  const entering = presentationDirection === "entering";
  const p =
    presentationDurationInFrames > 1
      ? Math.min(
          1,
          (presentationProgress * presentationDurationInFrames) /
            (presentationDurationInFrames - 1),
        )
      : 1;

  const radius = interpolate(p, [0, 1], [0, 1], CLAMPED);
  const scale = interpolate(p, [0, 1], [1, 2], CLAMPED);
  const thickness = interpolate(p, [0, 1], [1, 1.5], CLAMPED);

  const childStyle: React.CSSProperties = entering
    ? {
        opacity: interpolate(p, [0.55, 0.68], [0, 1], CLAMPED),
        scale: `${interpolate(p, [0.55, 1], [0.3, 1], {
          ...CLAMPED,
          easing: Easing.out(Easing.cubic),
        })}`,
        filter: `blur(${interpolate(p, [0.55, 0.75], [10, 0], CLAMPED)}px)`,
      }
    : { opacity: interpolate(p, [0.06, 0.13], [1, 0], CLAMPED) };

  return (
    <AbsoluteFill>
      {entering ? (
        <AbsoluteFill style={{ pointerEvents: "none" }}>
          <SmokeRingField
            speed={speed}
            colorBack={colorBack}
            colors={colors}
            radius={radius}
            scale={scale}
            thickness={thickness}
            noiseScale={detail}
            offsetX={(originX - 0.5) * 2}
          />
        </AbsoluteFill>
      ) : null}
      <AbsoluteFill style={childStyle}>{children}</AbsoluteFill>
    </AbsoluteFill>
  );
}

export function smokeDissolve(
  props: SmokeDissolveProps,
): TransitionPresentation<SmokeDissolveProps> {
  return {
    component: SmokeDissolvePresentation,
    props,
  };
}
