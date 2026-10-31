import { createContext, type CSSProperties, type ReactNode, useContext, useMemo } from "react";
import { AbsoluteFill } from "#runtime/parts.tsx";

export type Film = {
  width: number;
  height: number;
  fps: number;
  durationInFrames: number;
};

type Clock = Film & { frame: number };

const ClockContext = createContext<Clock | null>(null);

function useClock(): Clock {
  const clock = useContext(ClockContext);
  if (!clock) throw new Error("The film's clock is only there inside a film.");
  return clock;
}

export function FilmClock({
  film,
  frame,
  children,
}: {
  film: Film;
  frame: number;
  children?: ReactNode;
}) {
  const clock = useMemo(() => ({ ...film, frame }), [film, frame]);
  return <ClockContext.Provider value={clock}>{children}</ClockContext.Provider>;
}

export function useFrame(): number {
  return useClock().frame;
}

export function useFilm(): Film {
  const { width, height, fps, durationInFrames } = useClock();
  return useMemo(
    () => ({ width, height, fps, durationInFrames }),
    [width, height, fps, durationInFrames],
  );
}

function boundarySlack(frame: number, from: number, length: number): number {
  return Math.min(length / 2, Number.EPSILON * Math.max(Math.abs(frame), Math.abs(from)) * 4);
}

export type SequenceProps = {
  from?: number;
  durationInFrames?: number;
  layout?: "absolute-fill" | "none";
  width?: number;
  height?: number;
  name?: string;
  premountFor?: number;
  style?: CSSProperties;
  children?: ReactNode;
};

export function Sequence({
  from = 0,
  durationInFrames = Infinity,
  layout = "absolute-fill",
  width,
  height,
  style,
  children,
}: SequenceProps) {
  const parent = useClock();
  const frame = parent.frame - from;
  const length = Math.max(0, Math.min(durationInFrames, parent.durationInFrames - from));
  const clock = useMemo(
    () => ({
      ...parent,
      frame,
      durationInFrames: length,
      width: width ?? parent.width,
      height: height ?? parent.height,
    }),
    [parent, frame, length, width, height],
  );
  const slack = boundarySlack(parent.frame, from, durationInFrames);
  if (frame < -slack || frame - durationInFrames >= -slack) return null;
  return (
    <ClockContext.Provider value={clock}>
      {layout === "none" ? children : <AbsoluteFill style={style}>{children}</AbsoluteFill>}
    </ClockContext.Provider>
  );
}

export function Freeze({ frame, children }: { frame: number; children?: ReactNode }) {
  const parent = useClock();
  const clock = useMemo(() => ({ ...parent, frame }), [parent, frame]);
  return <ClockContext.Provider value={clock}>{children}</ClockContext.Provider>;
}

function loopStart(frame: number, length: number, total: number): number {
  const times = Math.ceil(total / length);
  const loops = frame / length;
  const nearest = Math.round(loops);
  const atBoundary = Math.abs(loops - nearest) <= Number.EPSILON * Math.max(1, Math.abs(loops)) * 4;
  const iteration = Math.max(0, Math.min(times - 1, atBoundary ? nearest : Math.floor(loops)));
  const start = atBoundary ? frame : iteration * length;
  return Math.max(0, Math.min(start, length * (times - 1)));
}

export function Loop({
  durationInFrames,
  children,
}: {
  durationInFrames: number;
  children?: ReactNode;
}) {
  const parent = useClock();
  return (
    <Sequence
      from={loopStart(parent.frame, durationInFrames, parent.durationInFrames)}
      durationInFrames={durationInFrames}
    >
      {children}
    </Sequence>
  );
}
