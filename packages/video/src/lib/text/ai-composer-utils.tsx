import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { spring } from "@opsis/runtime/spring.ts";
import type { CSSProperties } from "react";
import { CLAMPED } from "#video/core/anim.ts";
import { caretBlinkOpacity } from "#video/parts/caret.tsx";

const AI_TYPING_START = 42;
const AI_TYPING_START_TUI = 48;
const AI_TYPING_CPS = 22;

export function stageScale(width: number, height: number, refW = 1280, refH = 720): number {
  return Math.min(width / refW, height / refH);
}

interface TypewriterOptions {
  cps?: number;
  speed?: number;
  startFrame?: number;
}

interface TypewriterState {
  text: string;
  count: number;
  done: boolean;
  typing: boolean;
}

function useTypewriter(full: string, options: TypewriterOptions = {}): TypewriterState {
  const { cps = AI_TYPING_CPS, speed = 1, startFrame = AI_TYPING_START } = options;
  const frame = useFrame();
  const { fps } = useFilm();
  const local = frame * speed - startFrame;
  const over = (full.length / cps) * fps;
  const count =
    local <= 0
      ? 0
      : over <= 0
        ? full.length
        : Math.max(0, Math.min(full.length, Math.floor((local / over) * full.length)));

  return {
    text: full.slice(0, count),
    count,
    done: count >= full.length,
    typing: count > 0 && count < full.length,
  };
}

function morphProgressAt(
  frame: number,
  opts: { startFrame?: number; fps: number; speed?: number },
): number {
  const { startFrame = AI_TYPING_START, fps, speed = 1 } = opts;
  const value = spring({
    fps,
    frame: frame * speed - startFrame,
    config: { damping: 14, stiffness: 200, mass: 0.6 },
  });
  return Math.max(0, Math.min(value, 1));
}

interface SendBeat {
  frame: number;
  sent: boolean;
  press: number;
  bubble: number;
  reply: number;
}

function sendBeatAt(
  frame: number,
  opts: {
    promptLength: number;
    fps: number;
    cps?: number;
    startFrame?: number;
    pauseSeconds?: number;
  },
): SendBeat {
  const {
    promptLength,
    fps,
    cps = AI_TYPING_CPS,
    startFrame = AI_TYPING_START,
    pauseSeconds = 0.34,
  } = opts;
  const clampBoth = CLAMPED;
  const at = startFrame + (cps > 0 ? (promptLength / cps) * fps : 0) + pauseSeconds * fps;

  return {
    frame: at,
    sent: frame >= at,
    press: interpolate(frame, [at - 4, at, at + 8], [0, 1, 0], clampBoth),
    bubble: interpolate(frame, [at + 1, at + 15], [0, 1], {
      easing: Easing.out(Easing.cubic),
      ...clampBoth,
    }),
    reply: interpolate(frame, [at + 10, at + 22], [0, 1], {
      easing: Easing.out(Easing.cubic),
      ...clampBoth,
    }),
  };
}

function replyDotOpacity(frame: number, index: number, fps: number): number {
  const period = fps * 0.9;
  const phase = (((frame - index * (period / 3)) % period) + period) % period;
  return 0.28 + 0.72 * (0.5 - 0.5 * Math.cos((phase / period) * Math.PI * 2));
}

function introBounceIn(frame: number, fps: number): { translateY: number; scale: number } {
  const s = spring({
    fps,
    frame,
    config: { damping: 14, stiffness: 110, mass: 0.7 },
  });
  return {
    translateY: interpolate(s, [0, 1], [28, 0]),
    scale: interpolate(s, [0, 1], [0.97, 1], { output: "perceptual-scale" }),
  };
}

function fadeUpAt(
  frame: number,
  range: [number, number],
): { opacity: number; translateY: number } {
  const opts = CLAMPED;
  return {
    opacity: interpolate(frame, range, [0, 1], opts),
    translateY: interpolate(frame, range, [12, 0], opts),
  };
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
