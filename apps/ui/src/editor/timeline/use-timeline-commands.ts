import { useState } from "react";
import type { FlatTimeline, Removal, VideoBar } from "@/generated/bindings";
import { selectedSet } from "../../store/ui";
import {
  addMarkerAtPlayhead,
  commitEdit,
  hasSelectedMarker,
  removeNow,
  removeSelectedMarker,
  splitAtPlayhead,
} from "./timeline-actions";
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
  flat,
  sourceNodeId,
  zoom,
}: {
  flat: FlatTimeline;
  sourceNodeId: string | null;
  zoom: TimelineZoom;
}) {
  const [removal, setRemoval] = useState<Removal | null>(null);

  const remove = async () => {
    if (hasSelectedMarker()) return removeSelectedMarker();
    const next = await window.opsis.timelineRemoval([...selectedSet()]);
    if (!next) return;
    if (next.nodeIds.length > 1) setRemoval(next);
    else removeNow(next, sourceNodeId);
  };

  const commands: TimelineCommands = {
    split: () => splitAtPlayhead(selectedSet()),
    splitAt: (bar, frame) => commitEdit({ op: "splitBar", bar: bar.key, at: frame }),
    remove: () => void remove(),
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
