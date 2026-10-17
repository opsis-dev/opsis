import { nodeNameById } from "@/shared/nodes/registry";
import type { Workflow } from "@/shared/schema/workflow";
import { splitBar, type EditContext } from "@/shared/timeline/edits";
import type { FlatTimeline, VideoBar } from "@/shared/timeline/flatten";
import { commitEdit } from "./timeline-actions";
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

export function bladeSplit(ctx: BarContext, bar: VideoBar, clientX: number, alt: boolean): void {
  const at = ctx.drag.snapAt(ctx.drag.xToFrame(clientX), alt);
  commitEdit(splitBar(ctx.splitContext(), bar, Math.round(at)));
}

export function selectOnPress(ctx: BarContext, nodeId: string, shift: boolean): void {
  if (!ctx.selected.has(nodeId) || shift) ctx.select([nodeId], shift && ctx.tool === "select");
}
