import type { TransitionPresentation } from "@remotion/transitions";
import { clockWipe } from "@remotion/transitions/clock-wipe";
import { fade } from "@remotion/transitions/fade";
import { flip } from "@remotion/transitions/flip";
import { iris } from "@remotion/transitions/iris";
import { slide } from "@remotion/transitions/slide";
import { wipe } from "@remotion/transitions/wipe";

type Size = { width: number; height: number };
type Factory = (
  props: Record<string, unknown>,
  size: Size,
) => TransitionPresentation<Record<string, unknown>>;

export const coreTransitions: Record<string, Factory> = {
  "core/fade": () => fade() as never,
  "core/slide": (p) => slide({ direction: (p.direction as never) ?? "from-right" }) as never,
  "core/wipe": (p) => wipe({ direction: (p.direction as never) ?? "from-left" }) as never,
  "core/flip": (p) => flip({ direction: (p.direction as never) ?? "from-right" }) as never,
  "core/clock-wipe": (_p, size) => clockWipe(size) as never,
  "core/iris": (_p, size) => iris(size) as never,
};
