import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import type { CSSProperties } from "react";
import { caretBlinkOpacity } from "#video/parts/caret.tsx";

export function stageScale(width: number, height: number, refW = 1280, refH = 720): number {
  return Math.min(width / refW, height / refH);
}

export interface CaretProps {
  color?: string;
  width?: number;
  height?: number;
  radius?: number;
  opacity?: number;
  blink?: boolean;
  blinkPerSecond?: number;
  speed?: number;
  marginLeft?: number;
  style?: CSSProperties;
}

export function Caret({
  color = "currentColor",
  width = 2,
  height = 18,
  radius = 1,
  opacity,
  blink = false,
  blinkPerSecond = 1,
  speed = 1,
  marginLeft = 0,
  style,
}: CaretProps) {
  const frame = useFrame();
  const { fps } = useFilm();

  const resolvedOpacity =
    opacity !== undefined
      ? opacity
      : blink
        ? caretBlinkOpacity(frame, { fps, blinkPerSecond, speed })
        : 1;

  return (
    <span
      style={{
        display: "inline-block",
        flexShrink: 0,
        width,
        height,
        borderRadius: radius,
        background: color,
        opacity: resolvedOpacity,
        marginLeft,
        ...style,
      }}
    />
  );
}
