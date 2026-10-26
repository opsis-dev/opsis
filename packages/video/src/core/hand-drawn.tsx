import { Box, Circle, CrossedOff, Highlight, Underline } from "@remotion/rough-notation";
import React from "react";
import type { HandDrawnLook } from "#video/common/recording.ts";

export type MarkLook = HandDrawnLook | "crossed-off";

const ITERATIONS = 2;

type HandDrawnMarkProps = {
  look: MarkLook;
  progress: number;
  color: string;
  strokeWidth: number;
  seed: number;
  roughness?: number;
  children: React.ReactNode;
};

export const HandDrawnMark: React.FC<HandDrawnMarkProps> = ({
  look,
  strokeWidth,
  children,
  ...mark
}) => {
  const drawn = { ...mark, seed: Math.max(1, mark.seed), iterations: ITERATIONS };
  switch (look) {
    case "box":
      return (
        <Box {...drawn} strokeWidth={strokeWidth}>
          {children}
        </Box>
      );
    case "circle":
      return (
        <Circle {...drawn} strokeWidth={strokeWidth} box="around">
          {children}
        </Circle>
      );
    case "underline":
      return (
        <Underline {...drawn} strokeWidth={strokeWidth}>
          {children}
        </Underline>
      );
    case "crossed-off":
      return (
        <CrossedOff {...drawn} strokeWidth={strokeWidth}>
          {children}
        </CrossedOff>
      );
    case "marker":
      return <Highlight {...drawn}>{children}</Highlight>;
  }
};
