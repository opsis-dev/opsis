import { holdRender, useHeldLoad } from "@opsis/runtime/holds.ts";
import { useLayoutEffect, useRef, useState } from "react";
import type { Fit, ScreenProps } from "#video/generated/spec.ts";
import { decodeFrames, frameAt, type TerminalFrames } from "#video/terminal/frames.ts";
import {
  frameText,
  type Measure,
  screenSize,
  TERMINAL_FONT_STACK,
  TerminalView,
} from "#video/terminal/view.tsx";

type TerminalScreenProps = Extract<ScreenProps, { kind: "terminal" }>;

const BLINKS_PER_S = 1;

const ASPECT: Record<Fit, string> = {
  fill: "none",
  cover: "xMidYMid slice",
  contain: "xMidYMid meet",
};

async function loadFrames(url: string): Promise<TerminalFrames> {
  const response = await fetch(url);
  if (!response.ok)
    throw new Error(
      (await response.text()) || `Couldn't load the terminal take (${response.status}).`,
    );
  return decodeFrames(await response.arrayBuffer());
}

const widths = new Map<string, number>();
let canvas: CanvasRenderingContext2D | null = null;

function measurer(fontSize: number): Measure {
  return (text, bold) => {
    const key = `${bold ? 7 : 4}|${fontSize}|${text}`;
    const known = widths.get(key);
    if (known !== undefined) return known;
    canvas ??= document.createElement("canvas").getContext("2d");
    if (!canvas) return null;
    canvas.font = `${bold ? 700 : 400} ${fontSize}px ${TERMINAL_FONT_STACK}`;
    const width = canvas.measureText(text).width;
    widths.set(key, width);
    return width;
  };
}

function useFontsFor(text: string, fontSize: number): Measure | undefined {
  const [ready, setReady] = useState<string | null>(null);
  const holds = useRef(new Map<string, () => void>());
  const key = `${fontSize}|${text}`;
  useLayoutEffect(() => {
    if (!text || typeof document === "undefined") return;
    const release = holdRender("Loading the terminal's fonts");
    const open = holds.current;
    open.set(key, release);
    let alive = true;
    const loads = ["400", "700"].map((weight) =>
      document.fonts.load(`${weight} ${fontSize}px ${TERMINAL_FONT_STACK}`, text),
    );
    void Promise.allSettled(loads).then(() => {
      if (alive) setReady(key);
    });
    return () => {
      alive = false;
      open.delete(key);
      release();
    };
  }, [text, fontSize, key]);
  useLayoutEffect(() => {
    if (ready !== key) return;
    holds.current.get(key)?.();
    holds.current.delete(key);
  }, [ready, key]);
  return ready === key ? measurer(fontSize) : undefined;
}

export function cursorShown(timeS: number, blink: boolean): boolean {
  return !blink || Math.floor(timeS * BLINKS_PER_S * 2) % 2 === 0;
}

export function TerminalScreen({
  screen,
  timeS,
  fit,
}: {
  screen: TerminalScreenProps;
  timeS: number;
  fit: Fit;
}) {
  const loaded = useHeldLoad("Loading the terminal take", screen.framesUrl ?? null, loadFrames);
  const frames = loaded?.value;
  const frame = frames ? frameAt(frames, timeS) : undefined;
  const text = frames && frame ? frameText(frames, frame) : "";
  const measure = useFontsFor(text, screen.look.fontSize);
  if (!frames || !frame) return null;
  const size = screenSize(screen.look, frame.cols, frame.rows);
  return (
    <svg
      width="100%"
      height="100%"
      viewBox={`0 0 ${size.width} ${size.height}`}
      preserveAspectRatio={ASPECT[fit]}
      style={{ position: "absolute", inset: 0, display: "block" }}
    >
      <foreignObject x={0} y={0} width={size.width} height={size.height}>
        <TerminalView
          frames={frames}
          frame={frame}
          look={screen.look}
          cursorOn={cursorShown(timeS, screen.look.cursorBlink)}
          measure={measure}
        />
      </foreignObject>
    </svg>
  );
}
