import type React from "react";

export type PresentationDirection = "entering" | "exiting";

export type DrawFunction = (
  incoming: OffscreenCanvas | null,
  outgoing: OffscreenCanvas | null,
  progress: number,
) => void;

export type TransitionPresentationComponentProps<Props extends Record<string, unknown>> = {
  presentationProgress: number;
  children: React.ReactNode;
  presentationDirection: PresentationDirection;
  passedProps: Props;
  presentationDurationInFrames: number;
  onElementImage: (image: OffscreenCanvas, draw: DrawFunction) => void;
  onUnmount: () => void;
  bothEnteringAndExiting: boolean;
};

export type TransitionPresentation<Props extends Record<string, unknown>> = {
  component: (props: TransitionPresentationComponentProps<Props>) => React.ReactNode;
  props: Props;
};
