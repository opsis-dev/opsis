import type React from "react";
import type { ElementProps } from "#video/common/spec.ts";
import type { Size } from "#video/core/size.ts";
import type { TransitionPresentation } from "#video/lib/transitions/presentation.ts";

export type ElementComponent = React.ComponentType<ElementProps & { children?: React.ReactNode }>;

export type Presentation = TransitionPresentation<ElementProps>;

export type PresentationFactory = {
  readonly create: (props: ElementProps, size: Size) => Presentation;
};

type SceneDef = { durationInFrames: number; bg?: string; content: React.ReactNode };

export type SceneSequencer = {
  readonly Scenes: React.ComponentType<{ scenes: SceneDef[]; config: ElementProps }>;
  readonly Item: React.ComponentType<{
    index: number;
    style: React.CSSProperties;
    children: React.ReactNode;
  }>;
};

export type ElementImpl = ElementComponent | PresentationFactory | SceneSequencer;

export function isPresentation(impl: ElementImpl): impl is PresentationFactory {
  return "create" in impl;
}

export function isSequencer(impl: ElementImpl): impl is SceneSequencer {
  return "Scenes" in impl;
}
