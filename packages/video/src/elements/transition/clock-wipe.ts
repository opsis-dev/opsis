import { clockWipe as clockWipeOf } from "@remotion/transitions/clock-wipe";
import type { ElementProps } from "#video/common/spec.ts";
import type { Size } from "#video/core/still-layout.ts";
import { presentation } from "#video/registry/framing.tsx";

export const clockWipe = presentation((_props: ElementProps, size: Size) => clockWipeOf(size));
