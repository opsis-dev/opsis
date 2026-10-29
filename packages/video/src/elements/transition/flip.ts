import { flip as flipOf } from "@remotion/transitions/flip";
import type { ElementSettings } from "#video/generated/spec.ts";
import { presentation } from "#video/registry/framing.tsx";

export const flip = presentation(({ direction, perspective }: ElementSettings["flip"]) =>
  flipOf({ direction, perspective }),
);
