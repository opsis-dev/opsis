import { useState } from "react";
import { randomId } from "@/shared/ids";
import type { Workflow } from "@/shared/schema/workflow";
import { splitBar, type Removal } from "@/shared/timeline/edits";
import type { FlatTimeline, VideoBar } from "@/shared/timeline/flatten";
import { selectedSet } from "../../store/ui";
import {
  addMarkerAtPlayhead,
  commitEdit,
  hasSelectedMarker,
  removeNow,
  removeSelectedMarker,
  selectionRemoval,
  splitAtPlayhead,
} from "./timeline-actions";
import { editContext } from "./timeline-drag";
import { useTimeline } from "./timeline-state";
import type { TimelineZoom } from "./use-timeline-zoom";

export type TimelineCommands = {
  split: () => void;
  splitAt: (bar: VideoBar, frame: number) => void;
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
  const context = () => editContext(workflow, flat.fps, sourceNodeId, randomId());

  const remove = () => {
    if (hasSelectedMarker()) return removeSelectedMarker();
    const next = selectionRemoval(context(), flat, selectedSet());
    if (!next) return;
    if (next.nodeIds.length > 1) setRemoval(next);
    else removeNow(next, sourceNodeId);
  };

  const commands: TimelineCommands = {
    split: () => splitAtPlayhead(context(), flat, selectedSet()),
    splitAt: (bar, frame) => commitEdit(splitBar(context(), bar, frame)),
    remove,
    addMarker: () => addMarkerAtPlayhead(flat.fps),
    zoomBy: zoom.zoomAround,
    fit: useTimeline.getState().fitToView,
  };

  return {
    commands,
    removal,
    confirmRemoval: (r: Removal) => removeNow(r, sourceNodeId),
    closeRemoval: () => setRemoval(null),
  };
}
