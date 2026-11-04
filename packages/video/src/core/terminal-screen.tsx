import { holdRender, useHeldLoad } from "@opsis/runtime/holds.ts";
import { useId, useLayoutEffect, useRef, useState } from "react";
import type { Fit, ScreenProps } from "#video/generated/spec.ts";
import {
  EffectDefs,
  EffectLayers,
  hasLens,
  softenPx,
  trailWeights,
} from "#video/terminal/effects.tsx";
import { decodeFrames, frameAt, type TerminalFrames } from "#video/terminal/frames.ts";
import {
  frameText,
  type Measure,
  measurer,
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
      // oxlint-disable-next-line react/immutability
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
  const safeId = useId().replaceAll(":", "");
  if (!frames || !frame) return null;
  const size = screenSize(screen.look, frame.cols, frame.rows);
  const effects = screen.effects ?? undefined;
  const id = `terminal-${safeId}`;
  const cursorOn = cursorShown(timeS, screen.look.cursorBlink);
  const trails = trailWeights(effects).map((weight, k) => ({
    weight,
    frame: frameAt(frames, timeS - (k + 1) / frames.fps),
  }));
  const blur = softenPx(effects, size);
  return (
    <svg
      width="100%"
      height="100%"
      viewBox={`0 0 ${size.width} ${size.height}`}
      preserveAspectRatio={ASPECT[fit]}
      style={{ position: "absolute", inset: 0, display: "block" }}
    >
      {effects ? (
        <EffectDefs id={id} effects={effects} size={size} frame={Math.round(timeS * frames.fps)} />
      ) : null}
      <g filter={hasLens(effects) ? `url(#${id}-lens)` : undefined}>
        <foreignObject x={0} y={0} width={size.width} height={size.height}>
          <div style={{ position: "relative", filter: blur > 0 ? `blur(${blur}px)` : undefined }}>
            <TerminalView
              frames={frames}
              frame={frame}
              look={screen.look}
              cursorOn={cursorOn}
              measure={measure}
            />
            {trails.map(({ weight, frame: earlier }, k) =>
              earlier && earlier !== frame ? (
                <div
                  key={k}
                  style={{
                    position: "absolute",
                    inset: 0,
                    opacity: weight,
                    mixBlendMode: "lighten",
                  }}
                >
                  <TerminalView
                    frames={frames}
                    frame={earlier}
                    look={screen.look}
                    cursorOn={false}
                    measure={measure}
                  />
                </div>
              ) : null,
            )}
          </div>
        </foreignObject>
      </g>
      {effects ? <EffectLayers id={id} effects={effects} size={size} /> : null}
    </svg>
  );
}
