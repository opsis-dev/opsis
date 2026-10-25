import type { EditHandleKind, Slot } from "#core/schema/timeline";
import type { ChainLink, VideoBar } from "#core/timeline/flatten";

type SlotLink = ChainLink & { slot: Slot };

export const slotLink = (bar: Pick<VideoBar, "chain">): SlotLink | undefined =>
  bar.chain.find((l): l is SlotLink => !!l.slot);

export function slotIndex(bar: Pick<VideoBar, "chain">): number {
  const link = slotLink(bar);
  return link ? bar.chain.indexOf(link) : -1;
}

export function findHandle(
  bar: Pick<VideoBar, "chain">,
  kind: EditHandleKind,
  order: "inner" | "outer" = "inner",
): ChainLink | null {
  const limit = slotIndex(bar);
  const inScope = bar.chain.slice(0, limit === -1 ? bar.chain.length : limit + 1);
  const owns = (link: ChainLink) => !!link.handles?.[kind];
  return (order === "outer" ? inScope.findLast(owns) : inScope.find(owns)) ?? null;
}
