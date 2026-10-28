import { iris as irisOf } from "@remotion/transitions/iris";
import type { ElementProps } from "#video/common/spec.ts";
import type { Size } from "#video/core/still-layout.ts";
import { presentation } from "#video/registry/framing.tsx";

export const iris = presentation((_props: ElementProps, size: Size) => irisOf(size));
