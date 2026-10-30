import { AbsoluteFill } from "remotion";
import type { Direction, ElementSettings } from "#video/generated/spec.ts";
import type { TransitionPresentationComponentProps } from "#video/lib/transitions/presentation.ts";
import { presentation } from "#video/registry/framing.tsx";

type FlipProps = { direction: Direction; perspective: number };

const TURN = {
  "from-left": ["Y", -1],
  "from-right": ["Y", 1],
  "from-top": ["X", 1],
  "from-bottom": ["X", -1],
} as const satisfies Record<Direction, readonly ["X" | "Y", 1 | -1]>;

export function flipAngle(direction: Direction, entering: boolean, progress: number): number {
  const [, side] = TURN[direction];
  return entering ? side * 180 * (1 - progress) : -side * 180 * progress;
}

function Flip({
  children,
  presentationProgress,
  presentationDirection,
  passedProps: { direction, perspective },
}: TransitionPresentationComponentProps<FlipProps>) {
  const angle = flipAngle(direction, presentationDirection === "entering", presentationProgress);
  const [axis] = TURN[direction];
  return (
    <AbsoluteFill style={{ perspective }}>
      <AbsoluteFill
        style={{ transform: `rotate${axis}(${angle}deg)`, backfaceVisibility: "hidden" }}
      >
        {children}
      </AbsoluteFill>
    </AbsoluteFill>
  );
}

export const flip = presentation(({ direction, perspective }: ElementSettings["flip"]) => ({
  component: Flip,
  props: { direction, perspective },
}));
