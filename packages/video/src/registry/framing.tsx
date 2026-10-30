import type React from "react";
import type { ElementProps } from "#video/common/spec.ts";
import type { Size } from "#video/core/still-layout.ts";
import type { TransitionPresentation } from "#video/lib/transitions/presentation.ts";
import type {
  ElementComponent,
  Presentation,
  PresentationFactory,
  SceneSequencer,
} from "#video/registry/types.ts";

export function component(element: React.FC<never>): ElementComponent {
  return element as ElementComponent;
}

export function sequencer(Scenes: React.FC<never>, Item: React.FC<never>): SceneSequencer {
  return {
    Scenes: Scenes as SceneSequencer["Scenes"],
    Item: Item as SceneSequencer["Item"],
  };
}

export function presentation<P extends Record<string, unknown>>(
  make: (props: never, size: Size) => TransitionPresentation<P>,
): PresentationFactory {
  return {
    create: make as (props: ElementProps, size: Size) => Presentation,
  };
}
