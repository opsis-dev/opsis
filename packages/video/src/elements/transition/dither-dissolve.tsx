import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "@remotion/transitions";
import { AbsoluteFill, interpolate } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import { DitheringField } from "#video/parts/shader-fields.tsx";

const clampOpts = {
  extrapolateLeft: "clamp" as const,
  extrapolateRight: "clamp" as const,
};

const coverEnvelope = (p: number) => interpolate(p, [0, 0.3, 0.65, 1], [0, 1, 1, 0], clampOpts);

const coverChildOpacity = (p: number, entering: boolean) =>
  entering
    ? interpolate(p, [0.58, 0.66], [0, 1], clampOpts)
    : interpolate(p, [0.28, 0.36], [1, 0], clampOpts);

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
  const size = interpolate(p, [0, 1], [dotSize, dotSize * DOT_GROWTH], clampOpts);

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
