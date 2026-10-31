import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type React from "react";
import type { ElementSettings } from "#video/generated/spec.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";

type ZoomBlurProps = ElementSettings["zoom-blur"];

function ZoomBlurPresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<ZoomBlurProps>) {
  const { blur, rise, zoom } = passedProps;
  const entering = presentationDirection === "entering";
  const p = presentationProgress;

  const style: React.CSSProperties = entering
    ? {
        opacity: p,
        transform: `scale(${0.9 + p * 0.1})${rise > 0 ? ` translateY(${(1 - p) * rise}px)` : ""}`,
        filter: p < 1 ? `blur(${(1 - p) * blur}px)` : undefined,
      }
    : {
        transform: `scale(${1 + p * (zoom - 1)})${rise > 0 ? ` translateY(${-p * rise}px)` : ""}`,
        filter: p > 0 ? `blur(${p * blur}px)` : undefined,
      };

  return <AbsoluteFill style={style}>{children}</AbsoluteFill>;
}

export function zoomBlur(props: ZoomBlurProps): TransitionPresentation<ZoomBlurProps> {
  return {
    component: ZoomBlurPresentation,
    props,
  };
}
