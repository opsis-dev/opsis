import { useState } from "react";
import type { Workflow } from "@/shared/schema/workflow";
import type { Removal } from "@/shared/timeline/edits";
import type { FlatTimeline } from "@/shared/timeline/flatten";
import { useUiStore } from "../../store/ui";
import {
  addMarkerAtPlayhead,
  removeNow,
  removeSelectedMarker,
  selectionRemoval,
  splitAtPlayhead,
} from "./timeline-actions";
import { editContext, gestureSalt } from "./timeline-drag";
import { useTimeline } from "./timeline-state";
import type { TimelineZoom } from "./use-timeline-zoom";

export type TimelineCommands = {
  split: () => void;
  remove: () => void;
  addMarker: () => void;
  zoomBy: (factor: number) => void;
  fit: () => void;
};

export function useTimelineCommands({
  workflow,
  flat,
  sourceNodeId,
  zoom,
}: {
  workflow: Workflow;
  flat: FlatTimeline;
  sourceNodeId: string | null;
  zoom: TimelineZoom;
}) {
  const [removal, setRemoval] = useState<Removal | null>(null);
  const selected = () => new Set(useUiStore.getState().selectedIds);
  const context = (salt: string) => editContext(workflow, flat.fps, sourceNodeId, salt);

  const remove = () => {
    if (removeSelectedMarker()) return;
    const next = selectionRemoval(context("d"), flat, selected());
    if (!next) return;
    if (next.nodeIds.length > 1) setRemoval(next);
    else removeNow(next, sourceNodeId);
  };

  const commands: TimelineCommands = {
    split: () => splitAtPlayhead(context(gestureSalt()), flat, selected()),
    remove,
    addMarker: () => addMarkerAtPlayhead(flat.fps),
    zoomBy: (factor) => zoom.zoomAround(factor),
    fit: () => useTimeline.getState().fitToView(),
  };

  return {
    commands,
    removal,
    confirmRemoval: (r: Removal) => removeNow(r, sourceNodeId),
    closeRemoval: () => setRemoval(null),
  };
}
