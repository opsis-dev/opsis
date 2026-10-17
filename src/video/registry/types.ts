import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
import type { ElementProps } from "../../shared/elements/types";

export type ElementComponent = React.ComponentType<ElementProps & { children?: React.ReactNode }>;

export type PresentationFactory = (props: ElementProps) => TransitionPresentation<ElementProps>;

export type ElementImpl = ElementComponent | PresentationFactory;
