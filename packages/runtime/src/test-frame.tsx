import { type ComponentType, useMemo } from "react";
import { FilmClock } from "#runtime/clock.tsx";

type TestFrameProps<P extends object> = {
  component: ComponentType<P>;
  inputProps?: P;
  durationInFrames: number;
  fps: number;
  compositionWidth: number;
  compositionHeight: number;
  frameToDisplay: number;
};

export function TestFrame<P extends object>({
  component: Component,
  inputProps,
  durationInFrames,
  fps,
  compositionWidth: width,
  compositionHeight: height,
  frameToDisplay,
}: TestFrameProps<P>) {
  const film = useMemo(
    () => ({ width, height, fps, durationInFrames }),
    [width, height, fps, durationInFrames],
  );
  return (
    <FilmClock film={film} frame={frameToDisplay}>
      <Component {...(inputProps ?? ({} as P))} />
    </FilmClock>
  );
}
