import type { TransitionPresentation } from "@remotion/transitions";
import { useMemo } from "react";
import { AbsoluteFill } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import { transitionPhase } from "#video/lib/transitions/transition-timing.ts";

type BlindsProps = ElementSettings["transition-blinds"];

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

function slatProgress(coverage: number, index: number, count: number, stagger: number): number {
  if (count <= 1 || stagger <= 0) {
    return clamp01(coverage);
  }
  const spread = clamp01(stagger);
  const start = (index / (count - 1)) * spread;
  const span = 1 - spread;
  return clamp01((coverage - start) / (span > 0 ? span : 1));
}

function BlindsPresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps: { orientation, slats, stagger, alternate, edgeSoftness, reverse },
}: React.ComponentProps<NonNullable<TransitionPresentation<BlindsProps>["component"]>>) {
  const phase = transitionPhase(presentationProgress, presentationDirection, {
    lead: 1,
  });
  const coverage = 1 - phase.displace;
  const count = Math.max(1, Math.round(slats));

  const maskStyle = useMemo(() => {
    const horizontal = orientation === "horizontal";
    const feather = Math.max(edgeSoftness, 0) * 100;

    const layers = Array.from({ length: count }, (_, index) => {
      const progress = slatProgress(coverage, index, count, stagger);
      const reversed = (alternate && index % 2 === 1) !== reverse;
      const edge = progress * (100 + feather * 2) - feather;
      const direction = reversed
        ? horizontal
          ? "to left"
          : "to top"
        : horizontal
          ? "to right"
          : "to bottom";

      return `linear-gradient(${direction}, #000 ${(edge - feather).toFixed(2)}%, transparent ${(edge + feather).toFixed(2)}%)`;
    });

    const band = (100 / count) * 1.04;
    const size = horizontal ? `100% ${band.toFixed(4)}%` : `${band.toFixed(4)}% 100%`;

    const positions = Array.from({ length: count }, (_, index) => {
      const offset = count === 1 ? 0 : (index / (count - 1)) * 100;
      return horizontal ? `0% ${offset.toFixed(3)}%` : `${offset.toFixed(3)}% 0%`;
    });

    const image = layers.join(", ");
    const sizes = Array.from({ length: count }, () => size).join(", ");
    const places = positions.join(", ");

    return {
      WebkitMaskImage: image,
      maskImage: image,
      WebkitMaskSize: sizes,
      maskSize: sizes,
      WebkitMaskPosition: places,
      maskPosition: places,
      WebkitMaskRepeat: "no-repeat",
      maskRepeat: "no-repeat",
    };
  }, [alternate, count, coverage, edgeSoftness, orientation, reverse, stagger]);

  if (!phase.isEntering) {
    return <AbsoluteFill>{children}</AbsoluteFill>;
  }

  if (coverage >= 1) {
    return <AbsoluteFill>{children}</AbsoluteFill>;
  }

  return <AbsoluteFill style={maskStyle}>{children}</AbsoluteFill>;
}

export function blinds(props: BlindsProps): TransitionPresentation<BlindsProps> {
  return { component: BlindsPresentation, props };
}
