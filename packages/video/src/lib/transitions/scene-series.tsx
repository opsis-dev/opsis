import { Sequence, useFrame } from "@opsis/runtime/clock.tsx";
import React from "react";
import { CutBoardProvider, useCutBoard } from "#video/core/transition-cuts.tsx";
import type {
  PresentationDirection,
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";
import type { TransitionTiming } from "#video/lib/transitions/transition-timing.ts";

export type SeriesScene = { durationInFrames: number; content: React.ReactNode };

export type SeriesTransition<Props extends Record<string, unknown>> = {
  presentation: TransitionPresentation<Props>;
  timing: TransitionTiming;
};

type PlayedSide = Omit<TransitionPresentationComponentProps<Record<string, never>>, "passedProps">;

export type SeriesCut = {
  timing: TransitionTiming;
  play: (side: PlayedSide) => React.ReactNode;
};

export function seriesCut<Props extends Record<string, unknown>>({
  presentation,
  timing,
}: SeriesTransition<Props>): SeriesCut {
  const Presentation = presentation.component;
  return {
    timing,
    play: (side) => <Presentation {...side} passedProps={presentation.props} />,
  };
}

export function cutOverlaps(cuts: readonly (SeriesCut | null)[], count: number): number[] {
  return Array.from(
    { length: Math.max(0, count - 1) },
    (_, i) => cuts[i]?.timing.durationInFrames ?? 0,
  );
}

export function sceneStarts(lengths: readonly number[], overlaps: readonly number[]): number[] {
  let next = 0;
  return lengths.map((length, i) => {
    const start = next;
    next += length - (overlaps[i] ?? 0);
    return start;
  });
}

type Side = { cut: SeriesCut; direction: PresentationDirection; progress: number; at: number };

function sideAt(
  frame: number,
  length: number,
  { enter, exit }: { enter: SeriesCut | null; exit: SeriesCut | null },
): Side | null {
  if (enter && frame < enter.timing.durationInFrames)
    return { cut: enter, direction: "entering", progress: enter.timing.progressAt(frame), at: 0 };
  const exitStart = exit ? length - exit.timing.durationInFrames : length;
  if (exit && frame >= exitStart)
    return {
      cut: exit,
      direction: "exiting",
      progress: exit.timing.progressAt(frame - exitStart),
      at: exitStart,
    };
  return null;
}

function SceneInSeries({
  scene,
  start,
  enter,
  exit,
}: {
  scene: SeriesScene;
  start: number;
  enter: SeriesCut | null;
  exit: SeriesCut | null;
}) {
  const frame = useFrame();
  const board = useCutBoard();
  const side = sideAt(frame, scene.durationInFrames, { enter, exit });
  if (!side) return <>{scene.content}</>;
  const { cut, direction, progress } = side;
  const cutFrame = start + side.at;
  return cut.play({
    presentationProgress: progress,
    presentationDirection: direction,
    presentationDurationInFrames: cut.timing.durationInFrames,
    onElementImage: (image, draw) => board?.report(cutFrame, direction, { image, progress, draw }),
    onUnmount: () => board?.leave(cutFrame, direction),
    bothEnteringAndExiting: false,
    children: scene.content,
  });
}

export function SceneSeries({
  scenes,
  cuts,
}: {
  scenes: readonly SeriesScene[];
  cuts: readonly (SeriesCut | null)[];
}) {
  const overlaps = cutOverlaps(cuts, scenes.length);
  const starts = sceneStarts(
    scenes.map((scene) => scene.durationInFrames),
    overlaps,
  );
  return (
    <CutBoardProvider>
      {scenes.map((scene, i) => (
        <Sequence key={i} from={starts[i]} durationInFrames={scene.durationInFrames}>
          <SceneInSeries
            scene={scene}
            start={starts[i] ?? 0}
            enter={i > 0 ? (cuts[i - 1] ?? null) : null}
            exit={i < scenes.length - 1 ? (cuts[i] ?? null) : null}
          />
        </Sequence>
      ))}
    </CutBoardProvider>
  );
}
