import type { GroupProps, TimelineItem, TimelineTrack } from "#video/generated/spec.ts";

export const groupOf = (item: TimelineItem): GroupProps | null =>
  item.kind === "group" ? item.props : null;

export function walkItems(tracks: TimelineTrack[], visit: (item: TimelineItem) => void): void {
  for (const track of tracks)
    for (const item of track.items) {
      visit(item);
      if (item.kind === "group") walkItems(item.props.tracks, visit);
    }
}
