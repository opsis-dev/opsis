import { slide as slideOf } from "@remotion/transitions/slide";
import type { ElementSettings } from "#video/generated/spec.ts";
import { presentation } from "#video/registry/framing.tsx";

export const slide = presentation(({ direction }: ElementSettings["slide"]) =>
  slideOf({ direction }),
);
