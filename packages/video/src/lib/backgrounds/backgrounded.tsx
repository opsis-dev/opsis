import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import React from "react";

function visibleBackground(background?: string): string | undefined {
  return background && background !== "transparent" ? background : undefined;
}

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
