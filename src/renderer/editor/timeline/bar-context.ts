import type { Workflow } from "@/shared/schema/workflow";
import type { FlatTimeline, VideoBar } from "@/shared/timeline/flatten";
import type { Viewport } from "./timeline-layout";
import type { TimelineTool } from "./timeline-tools";
import type { TimelineDrag } from "./use-timeline-drag";

export type BarContext = {
  workflow: Workflow;
  flat: FlatTimeline;
  viewport: Viewport;
  tool: TimelineTool;
  selected: ReadonlySet<string>;
  drag: TimelineDrag;
  select: (ids: string[], additive: boolean) => void;
  splitAt: (bar: VideoBar, frame: number) => void;
};
