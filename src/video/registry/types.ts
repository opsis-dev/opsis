import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
import type { ElementProps } from "../../shared/elements/types";
import type { Size } from "../core/still-layout";

export type ElementComponent = React.ComponentType<ElementProps & { children?: React.ReactNode }>;

export type Presentation = TransitionPresentation<ElementProps>;

export type PresentationFactory = ((props: ElementProps, size: Size) => Presentation) & {
  readonly presentation: true;
};

export type ElementImpl = ElementComponent | PresentationFactory;

export const isPresentation = (impl: ElementImpl): impl is PresentationFactory =>
  "presentation" in impl;
