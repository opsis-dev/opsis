import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { ElementProps } from "#video/common/spec.ts";
import type { Size } from "#video/core/size.ts";
import type { TransitionPresentationComponentProps } from "#video/lib/transitions/presentation.ts";
import { presentation } from "#video/registry/framing.tsx";

type ClockProps = { width: number; height: number };

const STEP_DEGREES = 3;

export function clockPoints(
  { width, height }: ClockProps,
  progress: number,
): Array<[number, number]> {
  const reach = (1.5 * Math.hypot(width, height)) / 2;
  const sweep = progress * 360;
  const steps = Math.max(1, Math.ceil(sweep / STEP_DEGREES));
  const rim = Array.from({ length: steps + 1 }, (_, i): [number, number] => {
    const angle = ((-90 + (sweep * i) / steps) * Math.PI) / 180;
    return [width / 2 + reach * Math.cos(angle), height / 2 + reach * Math.sin(angle)];
  });
  return [[width / 2, height / 2], ...rim];
}

function ClockWipe({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<ClockProps>) {
  if (presentationDirection === "exiting") return <AbsoluteFill>{children}</AbsoluteFill>;
  const points = clockPoints(passedProps, presentationProgress)
    .map(([x, y]) => `${x}px ${y}px`)
    .join(", ");
  return <AbsoluteFill style={{ clipPath: `polygon(${points})` }}>{children}</AbsoluteFill>;
}

export const clockWipe = presentation((_props: ElementProps, { width, height }: Size) => ({
  component: ClockWipe,
  props: { width, height },
}));
