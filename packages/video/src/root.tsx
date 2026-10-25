import { EMPTY_SPEC, type TimelineSpec } from "@opsis/core/schema/timeline";
import React from "react";
import { Composition } from "remotion";
import { COMPOSITION_ID } from "#video/composition-id.ts";
import { DemoTimeline } from "#video/demo-timeline.tsx";

export const RemotionRoot: React.FC = () => (
  <Composition
    id={COMPOSITION_ID}
    component={DemoTimeline}
    defaultProps={EMPTY_SPEC}
    calculateMetadata={({ props }: { props: TimelineSpec }) => ({
      durationInFrames: Math.max(1, props.durationInFrames),
      fps: props.fps,
      width: props.width,
      height: props.height,
    })}
    durationInFrames={EMPTY_SPEC.durationInFrames}
    fps={EMPTY_SPEC.fps}
    width={EMPTY_SPEC.width}
    height={EMPTY_SPEC.height}
  />
);
