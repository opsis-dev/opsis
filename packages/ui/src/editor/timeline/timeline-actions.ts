import { randomId } from "@opsis/core/ids";
import type { Marker } from "@opsis/core/schema/workflow";
import type { FlatTimeline } from "@opsis/core/timeline/flatten";
import { framesToMs } from "@opsis/core/timeline/time";
import { toast } from "sonner";
import {
  applyRemoval,
  removalFor,
  splitBar,
  type EditContext,
  type EditResult,
  type Removal,
} from "@/timeline/edits";
import { usePlayback } from "../../store/playback";
import { toggleIds, useUiStore } from "../../store/ui";
import { useWorkflowStore } from "../../store/workflow";
import { useTimeline } from "./timeline-state";

export function selectNodes(ids: string[], additive: boolean, sourceNodeId: string | null): void {
  const ui = useUiStore.getState();
  ui.selectPinned(additive ? toggleIds(ui.selectedIds, ids) : ids, sourceNodeId);
  if (ids.length === 1) ui.revealNode(ids[0]);
}

export function reportEdit(r: EditResult): void {
  if (!r.ok) toast.error(r.error);
  else if (r.message) toast.success(r.message);
}

export function applyEdit(r: EditResult): void {
  if (r.ok) useWorkflowStore.getState().update(() => r.workflow);
}

export function commitEdit(r: EditResult): void {
  applyEdit(r);
  reportEdit(r);
}

export function splitAtPlayhead(
  ctx: EditContext,
  flat: FlatTimeline,
  selected: ReadonlySet<string>,
) {
  const at = usePlayback.getState().frame;
  const under = flat.bars.filter((b) => b.from < at && b.to > at);
  const bar =
    under.find((b) => selected.has(b.nodeId)) ?? under.toSorted((a, b) => b.lane - a.lane)[0];
  if (!bar) {
    toast.error("Put the playhead over a clip to split it.");
    return;
  }
  commitEdit(splitBar(ctx, bar, at));
}

export function selectionRemoval(
  ctx: EditContext,
  flat: FlatTimeline,
  selected: ReadonlySet<string>,
): Removal | null {
  const nodes = new Set<string>();
  const edges = new Set<string>();
  const bars = [
    ...flat.bars.filter((b) => selected.has(b.nodeId)),
    ...flat.audio.filter((a) => a.nodeId && selected.has(a.nodeId) && !a.bar),
  ];
  for (const bar of bars) {
    const r = removalFor(ctx, bar);
    r.nodeIds.forEach((id) => nodes.add(id));
    r.edgeIds.forEach((id) => edges.add(id));
  }
  if (!nodes.size && !edges.size) return null;
  return { nodeIds: [...nodes], edgeIds: [...edges] };
}

export function removeNow(removal: Removal, sourceNodeId: string | null): void {
  useWorkflowStore.getState().update((wf) => applyRemoval(wf, removal));
  useUiStore.getState().selectPinned([], sourceNodeId);
}

export const hasSelectedMarker = (): boolean => useTimeline.getState().selectedMarker !== null;

export function removeSelectedMarker(): void {
  const marker = useTimeline.getState().selectedMarker;
  useWorkflowStore.getState().setMarkers((list) => list.filter((m) => m.id !== marker));
  useTimeline.getState().selectMarker(null);
}

export function addMarkerAtPlayhead(fps: number): void {
  const at = usePlayback.getState().frame;
  const id = randomId("marker");
  useWorkflowStore
    .getState()
    .setMarkers((list) => [
      ...list,
      { id, atMs: framesToMs(at, fps), label: `Marker ${list.length + 1}` },
    ]);
  useTimeline.getState().selectMarker(id);
}

export function patchMarker(id: string, patch: Partial<Omit<Marker, "id">>): void {
  useWorkflowStore
    .getState()
    .setMarkers((list) => list.map((m) => (m.id === id ? { ...m, ...patch } : m)));
}
