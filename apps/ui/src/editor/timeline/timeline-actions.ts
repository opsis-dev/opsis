import { framesToMs } from "@opsis/video/common/time";
import { toast } from "sonner";
import { errorMessage } from "@/core/errors";
import type { Marker, Removal, TimelineEdit, TimelineReply } from "@/generated/bindings";
import { usePlayback } from "../../store/playback";
import { toggleIds, useUiStore } from "../../store/ui";
import { workflowEdits } from "../../store/workflow";
import { useTimeline } from "./timeline-state";

export function selectNodes(ids: string[], additive: boolean, sourceNodeId: string | null): void {
  const ui = useUiStore.getState();
  ui.selectPinned(additive ? toggleIds(ui.selectedIds, ids) : ids, sourceNodeId);
  if (ids.length === 1) ui.revealNode(ids[0]);
}

export const editTimeline = (edit: TimelineEdit): Promise<TimelineReply> =>
  window.opsis.editTimeline(edit).catch((e: unknown) => ({ error: errorMessage(e) }));

export function reportEdit(reply: TimelineReply): void {
  if (reply.error) toast.error(reply.error);
  else if (reply.message) toast.success(reply.message);
}

export const commitEdit = (edit: TimelineEdit): void => void editTimeline(edit).then(reportEdit);

export function splitAtPlayhead(selected: ReadonlySet<string>): void {
  commitEdit({ op: "split", at: usePlayback.getState().frame, selected: [...selected] });
}

export function removeNow(removal: Removal, sourceNodeId: string | null): void {
  workflowEdits.removeParts(removal);
  useUiStore.getState().selectPinned([], sourceNodeId);
}

export const hasSelectedMarker = (): boolean => useTimeline.getState().selectedMarker !== null;

export function removeSelectedMarker(): void {
  const marker = useTimeline.getState().selectedMarker;
  if (marker) workflowEdits.markers({ op: "remove", id: marker });
  useTimeline.getState().selectMarker(null);
}

export function addMarkerAtPlayhead(fps: number): void {
  const at = usePlayback.getState().frame;
  void workflowEdits
    .markers({ op: "add", atMs: framesToMs(at, fps) })
    .then((id) => id && useTimeline.getState().selectMarker(id));
}

export function patchMarker(id: string, patch: Partial<Omit<Marker, "id">>): void {
  workflowEdits.markers({ op: "update", id, ...patch });
}
