import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";
import { DitheringField } from "#video/parts/shader-fields.tsx";

function coverEnvelope(p: number) {
  return interpolate(p, [0, 0.3, 0.65, 1], [0, 1, 1, 0], CLAMPED);
}

function coverChildOpacity(p: number, entering: boolean) {
  return entering
    ? interpolate(p, [0.58, 0.66], [0, 1], CLAMPED)
    : interpolate(p, [0.28, 0.36], [1, 0], CLAMPED);
}

type DitherDissolveProps = ElementSettings["dither-dissolve"];

const DOT_GROWTH = 1.75;

function DitherDissolvePresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<DitherDissolveProps>) {
  const { colorBack, colorFront, shape, speed, dotSize, pattern } = passedProps;
  const entering = presentationDirection === "entering";
  const p = presentationProgress;
  const size = interpolate(p, [0, 1], [dotSize, dotSize * DOT_GROWTH], CLAMPED);

  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ opacity: coverChildOpacity(p, entering) }}>{children}</AbsoluteFill>
      {entering ? (
        <AbsoluteFill style={{ opacity: coverEnvelope(p), pointerEvents: "none" }}>
          <DitheringField
            speed={speed}
            colorBack={colorBack}
            colorFront={colorFront}
            shape={shape}
            type={pattern}
            size={size}
          />
        </AbsoluteFill>
      ) : null}
    </AbsoluteFill>
  );
}

export function ditherDissolve(
  props: DitherDissolveProps,
): TransitionPresentation<DitherDissolveProps> {
  return {
    component: DitherDissolvePresentation,
    props,
  };
}
