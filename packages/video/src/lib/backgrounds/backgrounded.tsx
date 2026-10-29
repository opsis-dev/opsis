import React from "react";
import { AbsoluteFill } from "remotion";

const visibleBackground = (background?: string): string | undefined =>
  background && background !== "transparent" ? background : undefined;

export function Backgrounded({
  background,
  children,
}: {
  background?: string;
  children?: React.ReactNode;
}) {
  return (
    <AbsoluteFill style={{ background: visibleBackground(background) }}>{children}</AbsoluteFill>
  );
}
