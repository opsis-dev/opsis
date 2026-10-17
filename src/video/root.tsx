import React from "react";
import { Composition } from "remotion";
import { EMPTY_SPEC, timelineSpecSchema, type TimelineSpec } from "../shared/schema/timeline";
import { COMPOSITION_ID } from "./composition-id";
import { DemoTimeline } from "./demo-timeline";

export const RemotionRoot: React.FC = () => (
  <Composition
    id={COMPOSITION_ID}
    component={DemoTimeline}
    schema={timelineSpecSchema}
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
