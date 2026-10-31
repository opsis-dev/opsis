import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { TransitionPresentationComponentProps } from "#video/lib/transitions/presentation.ts";
import { presentation } from "#video/registry/framing.tsx";

function Fade({
  children,
  presentationProgress,
  presentationDirection,
}: TransitionPresentationComponentProps<Record<string, never>>) {
  const opacity = presentationDirection === "entering" ? presentationProgress : 1;
  return <AbsoluteFill style={{ opacity }}>{children}</AbsoluteFill>;
}

export const fade = presentation(() => ({ component: Fade, props: {} }));
