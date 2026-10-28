import type { TransitionPresentation } from "@remotion/transitions";
import type { ElementProps } from "#video/common/spec.ts";
import { presentation } from "#video/registry/framing.tsx";
import type { PresentationFactory } from "#video/registry/types.ts";

export const EDGES = ["from-left", "from-top", "from-right", "from-bottom"] as const;
export const CORNERS = [
  "from-top-left",
  "from-top-right",
  "from-bottom-left",
  "from-bottom-right",
] as const;

export const directional = <D extends string, P extends Record<string, unknown>>(
  make: (props: { direction: D }) => TransitionPresentation<P>,
  directions: readonly D[],
  fallback: D,
): PresentationFactory =>
  presentation((props: ElementProps) =>
    make({ direction: directions.find((d) => d === props.direction) ?? fallback }),
  );
