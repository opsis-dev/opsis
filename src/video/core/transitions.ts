import type { TransitionPresentation } from "@remotion/transitions";
import { clockWipe } from "@remotion/transitions/clock-wipe";
import { fade } from "@remotion/transitions/fade";
import { flip } from "@remotion/transitions/flip";
import { iris } from "@remotion/transitions/iris";
import { slide } from "@remotion/transitions/slide";
import { wipe } from "@remotion/transitions/wipe";
import type { ElementProps } from "../../shared/elements/types";
import { presentation } from "../registry/framing";
import type { PresentationFactory } from "../registry/types";
import type { Size } from "./still-layout";

const EDGES = ["from-left", "from-top", "from-right", "from-bottom"] as const;
const CORNERS = [
  "from-top-left",
  "from-top-right",
  "from-bottom-left",
  "from-bottom-right",
] as const;

const directional = <D extends string, P extends Record<string, unknown>>(
  make: (props: { direction: D }) => TransitionPresentation<P>,
  directions: readonly D[],
  fallback: D,
): PresentationFactory =>
  presentation((props: ElementProps) =>
    make({ direction: directions.find((d) => d === props.direction) ?? fallback }),
  );

export const coreTransitions: Partial<Record<string, PresentationFactory>> = {
  "core/fade": presentation(() => fade()),
  "core/slide": directional(slide, EDGES, "from-right"),
  "core/wipe": directional(wipe, [...EDGES, ...CORNERS], "from-left"),
  "core/flip": directional(flip, EDGES, "from-right"),
  "core/clock-wipe": presentation((_props: ElementProps, size: Size) => clockWipe(size)),
  "core/iris": presentation((_props: ElementProps, size: Size) => iris(size)),
};
