import type { EditHandleKind, Slot } from "../schema/timeline";
import type { ChainLink, VideoBar } from "./flatten";

type HandleOwner = { link: ChainLink; index: number };

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
): HandleOwner | null {
  const limit = slotIndex(bar);
  const end = limit === -1 ? bar.chain.length : limit + 1;
  const step = order === "outer" ? -1 : 1;
  for (let i = order === "outer" ? end - 1 : 0; i >= 0 && i < end; i += step) {
    const link = bar.chain[i];
    if (link.handles?.[kind]) return { link, index: i };
  }
  return null;
}
