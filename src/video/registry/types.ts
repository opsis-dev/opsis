import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
import type { ElementProps } from "../../shared/elements/types";
import type { Size } from "../core/still-layout";

export type ElementComponent = React.ComponentType<ElementProps & { children?: React.ReactNode }>;

export type Presentation = TransitionPresentation<ElementProps>;

export type PresentationFactory = {
  readonly create: (props: ElementProps, size: Size) => Presentation;
};

export type ElementImpl = ElementComponent | PresentationFactory;

export const isPresentation = (impl: ElementImpl): impl is PresentationFactory => "create" in impl;
