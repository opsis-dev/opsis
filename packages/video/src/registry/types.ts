import type { ElementProps } from "@opsis/core/elements/types";
import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
import type { Size } from "#video/core/still-layout.ts";

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

export const isPresentation = (impl: ElementImpl): impl is PresentationFactory => "create" in impl;

export const isSequencer = (impl: ElementImpl): impl is SceneSequencer => "Scenes" in impl;
