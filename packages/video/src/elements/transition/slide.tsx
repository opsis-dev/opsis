import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { Direction, ElementSettings } from "#video/generated/spec.ts";
import type { TransitionPresentationComponentProps } from "#video/lib/transitions/presentation.ts";
import { presentation } from "#video/registry/framing.tsx";

type SlideProps = { direction: Direction };

const FROM = {
  "from-left": ["X", -1],
  "from-right": ["X", 1],
  "from-top": ["Y", -1],
  "from-bottom": ["Y", 1],
} as const satisfies Record<Direction, readonly ["X" | "Y", 1 | -1]>;

export function slideOffset(direction: Direction, entering: boolean, progress: number): number {
  const [, side] = FROM[direction];
  return entering ? side * (1 - progress) : -side * progress;
}

function Slide({
  children,
  presentationProgress,
  presentationDirection,
  passedProps: { direction },
}: TransitionPresentationComponentProps<SlideProps>) {
  const offset = slideOffset(direction, presentationDirection === "entering", presentationProgress);
  const [axis] = FROM[direction];
  return (
    <AbsoluteFill style={{ transform: `translate${axis}(${offset * 100}%)` }}>
      {children}
    </AbsoluteFill>
  );
}

export const slide = presentation(({ direction }: ElementSettings["slide"]) => ({
  component: Slide,
  props: { direction },
}));
