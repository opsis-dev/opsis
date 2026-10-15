import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
import type { ElementMeta, ElementProps } from "../../shared/elements/types";

export type ElementImpl =
  | React.ComponentType<ElementProps & { children?: React.ReactNode }>
  | ((props: ElementProps) => TransitionPresentation<ElementProps>);

export type ElementManifest = ElementMeta & { impl: ElementImpl };
