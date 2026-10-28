import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
import type { ElementProps } from "#video/common/spec.ts";
import type { Size } from "#video/core/still-layout.ts";
import type {
  ElementComponent,
  Presentation,
  PresentationFactory,
  SceneSequencer,
} from "#video/registry/types.ts";

export const component = (element: React.FC<never>): ElementComponent =>
  element as ElementComponent;

export const sequencer = (Scenes: React.FC<never>, Item: React.FC<never>): SceneSequencer => ({
  Scenes: Scenes as SceneSequencer["Scenes"],
  Item: Item as SceneSequencer["Item"],
});

export const presentation = <P extends Record<string, unknown>>(
  make: (props: never, size: Size) => TransitionPresentation<P>,
): PresentationFactory => ({
  create: make as (props: ElementProps, size: Size) => Presentation,
});
