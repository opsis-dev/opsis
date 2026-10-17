import { nodeNameById } from "@/shared/nodes/registry";
import type { Workflow } from "@/shared/schema/workflow";
import type { EditContext } from "@/shared/timeline/edits";
import type { FlatTimeline } from "@/shared/timeline/flatten";
import type { Viewport } from "./timeline-layout";
import type { TimelineTool } from "./timeline-state";
import type { TimelineDrag } from "./use-timeline-drag";

export type BarContext = {
  workflow: Workflow;
  flat: FlatTimeline;
  viewport: Viewport;
  tool: TimelineTool;
  selected: Set<string>;
  drag: TimelineDrag;
  select: (ids: string[], additive: boolean) => void;
  splitContext: () => EditContext;
};

export const nodeTitle = (wf: Workflow, id: string): string => nodeNameById(wf, id, "");
