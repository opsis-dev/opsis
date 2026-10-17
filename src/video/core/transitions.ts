import type { TransitionPresentation } from "@remotion/transitions";
import { clockWipe } from "@remotion/transitions/clock-wipe";
import { fade } from "@remotion/transitions/fade";
import { flip } from "@remotion/transitions/flip";
import { iris } from "@remotion/transitions/iris";
import { slide } from "@remotion/transitions/slide";
import { wipe } from "@remotion/transitions/wipe";

type Size = { width: number; height: number };
type Presentation = TransitionPresentation<Record<string, unknown>>;
type Factory = (props: Record<string, unknown>, size: Size) => Presentation;

const widen = <P extends Record<string, unknown>>(
  presentation: TransitionPresentation<P>,
): Presentation => presentation as unknown as Presentation;

const EDGES = ["from-left", "from-top", "from-right", "from-bottom"] as const;
const CORNERS = [
  "from-top-left",
  "from-top-right",
  "from-bottom-left",
  "from-bottom-right",
] as const;

const directional =
  <D extends string, P extends Record<string, unknown>>(
    make: (props: { direction: D }) => TransitionPresentation<P>,
    directions: readonly D[],
    fallback: D,
  ): Factory =>
  (props) =>
    widen(make({ direction: directions.find((d) => d === props.direction) ?? fallback }));

export const coreTransitions: Record<string, Factory> = {
  "core/fade": () => widen(fade()),
  "core/slide": directional(slide, EDGES, "from-right"),
  "core/wipe": directional(wipe, [...EDGES, ...CORNERS], "from-left"),
  "core/flip": directional(flip, EDGES, "from-right"),
  "core/clock-wipe": (_props, size) => widen(clockWipe(size)),
  "core/iris": (_props, size) => widen(iris(size)),
};
