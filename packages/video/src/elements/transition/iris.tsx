import { AbsoluteFill } from "remotion";
import type { ElementProps } from "#video/common/spec.ts";
import type { Size } from "#video/core/still-layout.ts";
import type { TransitionPresentationComponentProps } from "#video/lib/transitions/presentation.ts";
import { presentation } from "#video/registry/framing.tsx";

type IrisProps = { width: number; height: number };

export function irisRadius({ width, height }: IrisProps, progress: number): number {
  return (progress * Math.hypot(width, height)) / 2;
}

function Iris({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<IrisProps>) {
  if (presentationDirection === "exiting") return <AbsoluteFill>{children}</AbsoluteFill>;
  const radius = irisRadius(passedProps, presentationProgress);
  return (
    <AbsoluteFill style={{ clipPath: `circle(${radius}px at 50% 50%)` }}>{children}</AbsoluteFill>
  );
}

export const iris = presentation((_props: ElementProps, { width, height }: Size) => ({
  component: Iris,
  props: { width, height },
}));
