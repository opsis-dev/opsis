import React from "react";
import { AbsoluteFill } from "remotion";

const visibleBackground = (background?: string): string | undefined =>
  background && background !== "transparent" ? background : undefined;

export const Backgrounded: React.FC<{ background?: string; children?: React.ReactNode }> = ({
  background,
  children,
}) => <AbsoluteFill style={{ background: visibleBackground(background) }}>{children}</AbsoluteFill>;
