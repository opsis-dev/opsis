import type { FilmDrawing } from "@opsis/runtime/page.tsx";
import { DemoTimeline } from "#video/demo-timeline.tsx";
import type { TimelineSpec } from "#video/generated/spec.ts";
import "#video/video.css";

export const FILM: FilmDrawing<TimelineSpec> = {
  film(spec) {
    return {
      width: spec.width,
      height: spec.height,
      fps: spec.fps,
      durationInFrames: Math.max(1, spec.durationInFrames),
    };
  },
  draw(spec) {
    return <DemoTimeline {...spec} />;
  },
};
